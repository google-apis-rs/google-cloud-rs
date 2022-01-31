use std::borrow::Borrow;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use tonic::{IntoRequest, Request};

use crate::authorize::{ApplicationCredentials, TokenManager, TLS_CERTS};
use crate::datastore::api;
use crate::datastore::api::datastore_client::DatastoreClient;
use crate::datastore::{
    Entity, Error, Filter, FromValue, IntoEntity, Key, KeyID, Order, Query, Value,
};

use super::{Transaction, IndexExcluded};
use super::api::transaction_options::{ReadWrite, ReadOnly};

/// The Datastore client, tied to a specific project.
#[derive(Clone)]
pub struct Client {
    pub(crate) project_name: String,
    pub(crate) service: DatastoreClient<Channel>,
    pub(crate) token_manager: Arc<Mutex<TokenManager>>,
    pub(crate) index_excluded: IndexExcluded,
}

/// Opciones para el modo de crear la trx
#[derive(Debug, Clone, PartialEq)]
pub enum TrxOption {
    /// modo solo lectura
    ReadOnly,
    /// modo de escritura y lectura
    ReadWrite,
    /// modo por defecto 
    Default,
}

impl Client {
    pub(crate) const DOMAIN_NAME: &'static str = "datastore.googleapis.com";
    pub(crate) const ENDPOINT: &'static str = "https://datastore.googleapis.com";
    pub(crate) const SCOPES: [&'static str; 2] = [
        "https://www.googleapis.com/auth/cloud-platform",
        "https://www.googleapis.com/auth/datastore",
    ];

    pub(crate) async fn construct_request<T: IntoRequest<T>>(
        &mut self,
        request: T,
    ) -> Result<Request<T>, Error> {
        let mut request = request.into_request();
        let token = self.token_manager.lock().await.token().await?;
        let metadata = request.metadata_mut();
        metadata.insert("authorization", token.parse().unwrap());
        Ok(request)
    }

    /// Creates a new client for the specified project.
    ///
    /// Credentials are looked up in the `GOOGLE_APPLICATION_CREDENTIALS` environment variable.
    pub async fn new(project_name: impl Into<String>) -> Result<Client, Error> {
        let path = env::var("GOOGLE_APPLICATION_CREDENTIALS")?;
        let file = File::open(path)?;
        let creds = json::from_reader(file)?;

        Client::from_credentials(project_name, creds).await
    }

    /// Creates a new client for the specified project with custom credentials.
    pub async fn from_credentials(
        project_name: impl Into<String>,
        creds: ApplicationCredentials,
    ) -> Result<Client, Error> {
        let tls_config = ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(TLS_CERTS))
            .domain_name(Client::DOMAIN_NAME);

        let channel = Channel::from_static(Client::ENDPOINT)
            .tls_config(tls_config)?
            .connect()
            .await?;

        Ok(Client {
            project_name: project_name.into(),
            service: DatastoreClient::new(channel),
            token_manager: Arc::new(Mutex::new(TokenManager::new(
                creds,
                Client::SCOPES.as_ref(),
            ))),
            index_excluded: IndexExcluded::new()?,
        })
    }

    /// Create a new transaction
    ///     - option_mode: Option for the transaction
    ///     - trx_id: Clave de la transacción anterior y que por algún motivo fallo y se ejecuto el rollback
    pub async fn new_transaction(&mut self, option_mode: TrxOption, trx_id: Option<Vec<u8>>) -> Result<Transaction, Error> {
        let trx_option = match option_mode {
            TrxOption::ReadOnly => Some(api::TransactionOptions {
                            mode: Some(api::transaction_options::Mode::ReadOnly(ReadOnly{}))
                        }),
            TrxOption::ReadWrite => match trx_id {
                Some(trx) => Some(api::TransactionOptions {
                    mode: Some(api::transaction_options::Mode::ReadWrite(ReadWrite {previous_transaction: trx}))
                }),
                None => None,
            },
            TrxOption::Default => None,
        };

        let request = api::BeginTransactionRequest {
            project_id: self.project_name.clone(),
            transaction_options: trx_option,
        };

        let request = self.construct_request(request).await?;
        let response = self.service.begin_transaction(request).await?;
        let response = response.into_inner();

        Ok(Transaction::new(self.to_owned(), response.transaction))
    }

    /// Gets an entity from a key.
    pub async fn get<T, K>(&mut self, key: K) -> Result<Option<T>, Error>
    where
        K: Borrow<Key>,
        T: FromValue,
    {
        let results = self.get_all(Some(key.borrow())).await?;
        Ok(results.into_iter().next().map(T::from_value).transpose()?)
    }

    /// Gets multiple entities from multiple keys.
    pub async fn get_all<T, K, I>(&mut self, keys: I) -> Result<Vec<T>, Error> 
    where
        I: IntoIterator<Item = K>,
        K: Borrow<Key>,
        T: FromValue,
    {
        Ok(self.get_all_tx(keys, None).await?)
    }

    /// Gets multiple entities from multiple keys associated with a transaction
    pub(crate) async fn get_all_tx<T, K, I>(&mut self, keys: I, tx_id: Option<Vec<u8>>) -> Result<Vec<T>, Error>
    where
        I: IntoIterator<Item = K>,
        K: Borrow<Key>,
        T: FromValue,
    {
        let og_keys: Vec<K> = keys.into_iter().collect();
        let mut keys: Vec<_> = og_keys
            .iter()
            .map(|key| convert_key(self.project_name.as_str(), key.borrow()))
            .collect();
        let mut found = HashMap::new();

        while !keys.is_empty() {
            let request = match tx_id.to_owned() {
                Some(tx) => api::LookupRequest {
                    keys, 
                    project_id: self.project_name.clone(), 
                    read_options: Some(api::ReadOptions {
                        consistency_type: Some(api::read_options::ConsistencyType::Transaction(tx)),
                    }),
                },
                None => api::LookupRequest {
                    keys,
                    project_id: self.project_name.clone(),
                    read_options: None,
                }
            };

            let request = self.construct_request(request).await?;
            let response = self.service.lookup(request).await?;
            
            let response = response.into_inner();
            found.extend(
                response
                    .found
                    .into_iter()
                    .map(|val| val.entity.unwrap())
                    .map(Entity::from)
                    .map(|entity| (entity.key, entity.properties)),
            );
            keys = response.deferred;
        }

        let values: Vec<T> = og_keys
            .into_iter()
            .flat_map(|key| found.remove(key.borrow()))
            .map(FromValue::from_value)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(values)
    }

    /// Inserts a new entity and returns its key.
    /// If the entity's key is incomplete, the returned key will be one generated by the store for this entity.
    pub async fn put(&mut self, entity: impl IntoEntity) -> Result<Option<Key>, Error> {
        let entity = entity.into_entity()?;
        let result = self.put_all(Some(entity)).await?;
        Ok(result.into_iter().next().flatten())
    }

    /// Inserts new entities and returns their keys.
    /// If an entity's key is incomplete, its returned key will be one generated by the store for this entity.
    pub async fn put_all<T, I>(&mut self, entities: I) -> Result<Vec<Option<Key>>, Error>
    where
        I: IntoIterator<Item = T>,
        T: IntoEntity,
    {
        let entities: Vec<Entity> = entities
            .into_iter()
            .map(IntoEntity::into_entity)
            .collect::<Result<_, _>>()?;

        let mutations = entities
            .into_iter()
            .map(|entity| {
                let is_incomplete = entity.key.is_new || entity.key.is_incomplete();
                let entity = convert_entity(self.project_name.as_str(), entity, self.index_excluded.to_owned());
                api::Mutation {
                    operation: if is_incomplete {
                        Some(api::mutation::Operation::Insert(entity))
                    } else {
                        Some(api::mutation::Operation::Upsert(entity))
                    },
                    conflict_detection_strategy: None,
                }
            })
            .collect();

        let request = api::CommitRequest {
            mutations,
            mode: api::commit_request::Mode::NonTransactional as i32,
            transaction_selector: None,
            project_id: self.project_name.clone(),
        };
        let request = self.construct_request(request).await?;
        let response = self.service.commit(request).await?;
        let response = response.into_inner();
        let keys = response
            .mutation_results
            .into_iter()
            .map(|result| result.key.map(Key::from))
            .collect();

        Ok(keys)
    }

    /// Deletes an entity identified by a key.
    pub async fn delete(&mut self, key: impl Borrow<Key>) -> Result<(), Error> {
        self.delete_all(Some(key.borrow())).await
    }

    /// Deletes multiple entities identified by multiple keys.
    pub async fn delete_all<T, I>(&mut self, keys: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = T>,
        T: Borrow<Key>,
    {
        let mutations = keys
            .into_iter()
            .map(|key| convert_key(self.project_name.as_str(), key.borrow()))
            .map(|key| api::Mutation {
                operation: Some(api::mutation::Operation::Delete(key)),
                conflict_detection_strategy: None,
            })
            .collect();

        let request = api::CommitRequest {
            mutations,
            mode: api::commit_request::Mode::NonTransactional as i32,
            transaction_selector: None,
            project_id: self.project_name.clone(),
        };
        let request = self.construct_request(request).await?;
        self.service.commit(request).await?;

        Ok(())
    }

    /// Runs a (potentially) complex query againt Datastore and returns the results.
    pub async fn query(&mut self, query: Query) -> Result<Vec<Entity>, Error> {
        Ok(self.query_tx(query, None).await?)
    }

    /// Runs a (potentially) complex query againt Datastore and returns the results and associated with a transaction
    pub(crate) async fn query_tx(&mut self, query: Query, tx_id: Option<Vec<u8>>) -> Result<Vec<Entity>, Error> {
        let mut output = Vec::new();

        let mut cur_query = query.clone();
        let mut cursor = Vec::new();
        loop {
            let projection = cur_query
                .projections
                .into_iter()
                .map(|name| api::Projection {
                    property: Some(api::PropertyReference { name }),
                })
                .collect();
            let filter = convert_filter(self.project_name.as_str(), cur_query.filters);
            let order = cur_query
                .ordering
                .into_iter()
                .map(|order| {
                    use api::property_order::Direction;
                    let (name, direction) = match order {
                        Order::Asc(name) => (name, Direction::Ascending),
                        Order::Desc(name) => (name, Direction::Descending),
                    };
                    api::PropertyOrder {
                        property: Some(api::PropertyReference { name }),
                        direction: direction as i32,
                    }
                })
                .collect();
            let api_query = api::Query {
                kind: vec![api::KindExpression {
                    name: cur_query.kind,
                }],
                projection,
                filter,
                order,
                offset: cur_query.offset,
                limit: cur_query.limit,
                start_cursor: cursor,
                end_cursor: Vec::new(),
                distinct_on: cur_query
                    .distinct_on
                    .into_iter()
                    .map(|name| api::PropertyReference { name })
                    .collect(),
            };
            let request = api::RunQueryRequest {
                partition_id: Some(api::PartitionId {
                    project_id: self.project_name.clone(),
                    namespace_id: cur_query.namespace.unwrap_or_else(String::new),
                }),
                query_type: Some(api::run_query_request::QueryType::Query(api_query)),
                read_options: Some({
                    use api::read_options::{ConsistencyType, ReadConsistency};
                    api::ReadOptions {
                        consistency_type: Some(
                            match tx_id.to_owned() {
                                Some(tx) => ConsistencyType::Transaction(tx),
                                None => ConsistencyType::ReadConsistency(
                                    if cur_query.eventual {
                                        ReadConsistency::Eventual as i32
                                    } else {
                                        ReadConsistency::Strong as i32
                                    },
                                ),
                            }
                        ),
                    }
                }),
                project_id: self.project_name.clone(),
            };
            let request = self.construct_request(request).await?;
            let results = self.service.run_query(request).await?;
            let results = results.into_inner().batch.unwrap();

            output.extend(
                results
                    .entity_results
                    .into_iter()
                    .map(|el| Entity::from(el.entity.unwrap())),
            );

            if results.more_results
                != (api::query_result_batch::MoreResultsType::NotFinished as i32)
            {
                break Ok(output);
            }

            cur_query = query.clone();
            cursor = results.end_cursor;
        }
    }
}

pub(crate) fn convert_key(project_name: &str, key: &Key) -> api::Key {
    api::Key {
        partition_id: Some(api::PartitionId {
            project_id: String::from(project_name),
            namespace_id: key.get_namespace().map(String::from).unwrap_or_default(),
        }),
        path: {
            let mut key = Some(key);
            let mut path = Vec::new();
            while let Some(current) = key {
                path.push(api::key::PathElement {
                    kind: String::from(current.get_kind()),
                    id_type: match current.get_id() {
                        KeyID::Incomplete => None,
                        KeyID::IntID(id) => Some(api::key::path_element::IdType::Id(*id)),
                        KeyID::StringID(id) => {
                            Some(api::key::path_element::IdType::Name(id.clone()))
                        }
                    },
                });
                key = current.get_parent();
            }
            path.reverse();
            path
        },
    }
}

pub(crate) fn convert_entity(project_name: &str, entity: Entity, index_excluded: IndexExcluded) -> api::Entity {
    let key = convert_key(project_name, &entity.key);
    let properties = match entity.clone().properties {
        Value::EntityValue(properties) => properties,
        _ => panic!("unexpected non-entity datastore value"),
    };
    let properties = properties
        .into_iter()
        .map(|(k, v)| {
            let index_excluded = IndexExcluded::ckeck_value(index_excluded.to_owned(), entity.key.get_kind().to_owned(), k.to_owned());
            (k, convert_value(project_name, v, index_excluded))
        }
        ).collect();
    api::Entity {
        key: Some(key),
        properties,
    }
}

pub(crate) fn convert_value(project_name: &str, value: Value, index_excluded: bool) -> api::Value {
    let value_type = match value {
        Value::NULL(_) => api::value::ValueType::NullValue(0),
        Value::BooleanValue(val) => api::value::ValueType::BooleanValue(val),
        Value::IntegerValue(val) => api::value::ValueType::IntegerValue(val),
        Value::DoubleValue(val) => api::value::ValueType::DoubleValue(val),
        Value::TimestampValue(val) => api::value::ValueType::TimestampValue(prost_types::Timestamp {
            seconds: val.timestamp(),
            nanos: val.timestamp_subsec_nanos() as i32,
        }),
        Value::KeyValue(key) => api::value::ValueType::KeyValue(convert_key(project_name, &key)),
        Value::StringValue(val) => api::value::ValueType::StringValue(val),
        Value::BlobValue(val) => api::value::ValueType::BlobValue(val),
        Value::GeoPointValue(latitude, longitude) => api::value::ValueType::GeoPointValue(api::LatLng {
            latitude,
            longitude,
        }),
        Value::EntityValue(properties) => api::value::ValueType::EntityValue({
            api::Entity {
                key: None,
                properties: properties
                    .into_iter()
                    .map(|(k, v)| (k, convert_value(project_name, v, index_excluded)))
                    .collect(),
            }
        }),
        Value::ArrayValue(values) => api::value::ValueType::ArrayValue(api::ArrayValue {
            values: values
                .into_iter()
                .map(|value| convert_value(project_name, value, index_excluded))
                .collect(),
        }),
    };
    api::Value {
        meaning: 0,
        exclude_from_indexes: index_excluded,
        value_type: Some(value_type),
    }
}

pub(crate) fn convert_filter(project_name: &str, filters: Vec<Filter>) -> Option<api::Filter> {
    use api::filter::FilterType;

    if !filters.is_empty() {
        let filters = filters
            .into_iter()
            .map(|filter| {
                use api::property_filter::Operator;
                let (name, op, value) = match filter {
                    Filter::Equal(name, value) => (name, Operator::Equal, value),
                    Filter::GreaterThan(name, value) => (name, Operator::GreaterThan, value),
                    Filter::LesserThan(name, value) => (name, Operator::LessThan, value),
                    Filter::GreaterThanOrEqual(name, value) => {
                        (name, Operator::GreaterThanOrEqual, value)
                    }
                    Filter::LesserThanEqual(name, value) => {
                        (name, Operator::LessThanOrEqual, value)
                    }
                    Filter::HasAncestor(value) => {
                        ("__key__".to_string(), Operator::HasAncestor, value)
                    }
                };

                api::Filter {
                    filter_type: Some(FilterType::PropertyFilter(api::PropertyFilter {
                        op: op as i32,
                        property: Some(api::PropertyReference { name }),
                        value: Some(convert_value(project_name, value, false)),
                    })),
                }
            })
            .collect();

        Some(api::Filter {
            filter_type: Some(FilterType::CompositeFilter(api::CompositeFilter {
                op: api::composite_filter::Operator::And as i32,
                filters,
            })),
        })
    } else {
        None
    }
}
