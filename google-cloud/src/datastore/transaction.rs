use std::borrow::Borrow;
use super::{Client, api::{CommitRequest, self, Mutation, RollbackRequest}, FromValue, Key, convert_key, convert_entity, Query};
use crate::datastore::{
    Entity, Error, IntoEntity};

/// Structure where the data necessary to manage the transaction is stored
///     - client: The Datastore client
///     - tx_key: key returned by google cloud datastore to identify the Transaction
///     - commit_request: Where we accumulate the mutations
#[derive(Clone)]
pub struct Transaction {
    pub(crate) client: Client,
    pub(crate) tx_key: Vec<u8>,
    pub(crate) commit_request: CommitRequest,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(client: Client, tx_key: Vec<u8>) -> Transaction {
        let project_name = client.clone().project_name;

        Transaction {
            client,
            tx_key: tx_key.to_vec(),
            commit_request: api::CommitRequest {
                mutations: Vec::new(),
                mode: api::commit_request::Mode::Transactional as i32,
                transaction_selector: Some(api::commit_request::TransactionSelector::Transaction(tx_key.to_vec())),
                project_id: project_name,
            },
        }
    }

    /// Gets an entity from a key associated with a transaction
    pub async fn get<T, K>(&mut self, key: K) -> Result<Option<T>, Error>
    where
        K: Borrow<Key>,
        T: FromValue,
    {
        let results = self.get_all(Some(key.borrow())).await?;
        Ok(results.into_iter().next().map(T::from_value).transpose()?)
    }

    /// Gets multiple entities from multiple keys associated with a transaction
    pub async fn get_all<T, K, I>(&mut self, keys: I) -> Result<Vec<T>, Error>
    where
        I: IntoIterator<Item = K>,
        K: Borrow<Key>,
        T: FromValue,
    {
        Ok(self.client.get_all_tx(keys, Some(self.tx_key.to_vec())).await?)
    }

    /// Create, Modify or delete entity and returns its key.
    /// the Key can be marked as:
    ///     - newId
    ///     - delete
    /// If id is not indicated, it will be incomplete and, by default, a new entity will be created.
    /// 
    /// This method can be called more than once for the same transaction, because the different 
    /// types of mutations are accumulated to subsequently execute the commit, which will send all 
    /// the information and return the Datastore response.
    /// 
    /// Different types of mutations can be mixed in the same transaction (creation, modification and deletion)
    pub async fn put(&mut self, entity: impl IntoEntity) -> Result<(), Error> {
        let entity = entity.into_entity()?;
        self.put_all(Some(entity)).await?;
        Ok(())
    }

    /// Same operation as the put method but with multiple entities.
    pub async fn put_all<T, I>(&mut self, entities: I) -> Result<(), Error>
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
                let operation = match entity.key.delete {
                    true => {
                        let key = convert_key(self.client.project_name.as_str(), entity.key.borrow());
                        Some(api::mutation::Operation::Delete(key))
                    },
                    false => {
                        let is_incomplete = entity.key.is_new || entity.key.is_incomplete();
                        let entity = convert_entity(self.client.project_name.as_str(), entity, self.client.index_excluded.to_owned());
                        match is_incomplete {
                            true => Some(api::mutation::Operation::Insert(entity)),
                            false => Some(api::mutation::Operation::Upsert(entity)),
                        }
                    },
                };
                api::Mutation {
                    operation,
                    conflict_detection_strategy: None,
                }
            })
            .collect::<Vec<Mutation>>();

        self.commit_request.mutations.append(&mut mutations.to_vec());

        Ok(())
    }

    /// Execute a (potentially) complex query against the Datastore 
    /// in a transaction and return the results.
    pub async fn query(&mut self, query: Query) -> Result<Vec<Entity>, Error> {
        Ok(self.client.query_tx(query, Some(self.tx_key.to_vec())).await?)
    }

    /// Execute the transaction with the accumulated information.
    /// Note that delete mutations do not return anything.
    pub async fn commit(&mut self) -> Result<Vec<Option<Key>>, Error> {
        let request = self.client.construct_request(self.commit_request.to_owned()).await?;
        let response = self.client.service.commit(request).await?;

        let response = response.into_inner();
        let keys = response
            .mutation_results
            .into_iter()
            .map(|result| result.key.map(Key::from))
            .collect();

        Ok(keys)
    }

    /// Execute transaction rollback
    pub async fn rollback(&mut self) -> Result<(), Error> {
        self.client.service.rollback(
            RollbackRequest {
                project_id: self.client.project_name.to_owned(),
                transaction: self.tx_key.to_vec()
            }
        ).await?;
        
        Ok(())
    }
}
