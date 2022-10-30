use crate::datastore::Value;
use super::{IntoValue, Key};

/// Represents Datastore query result orderings.
#[derive(Debug, Clone, PartialEq)]
pub enum Order {
    /// Ascendent ordering.
    Asc(String),
    /// Descendent ordering.
    Desc(String),
}

/// Represents Datastore query result orderings.
#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    /// Equality filter (==).
    Equal(String, Value),
    /// Greater-than filter (>).
    GreaterThan(String, Value),
    /// Lesser-than filter (<).
    LesserThan(String, Value),
    /// Greater-than-or-equal filter (>=).
    GreaterThanOrEqual(String, Value),
    /// Lesser-than-or-equal filter (<=).
    LesserThanEqual(String, Value),
    /// Append ancestor to the Query
    HasAncestor(Value),
    /// In
    In(String, Value),
    /// NotIn
    NotIn(String, Value),
    /// NotEqual
    NotEqual(String, Value),
}

/// Represents a Datastore query.
#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub(crate) kind: String,
    pub(crate) eventual: bool,
    pub(crate) keys_only: bool,
    pub(crate) offset: i32,
    pub(crate) limit: Option<i32>,
    pub(crate) namespace: Option<String>,
    pub(crate) projections: Vec<String>,
    pub(crate) distinct_on: Vec<String>,
    pub(crate) ordering: Vec<Order>,
    pub(crate) filters: Vec<Filter>,
    pub(crate) cursor: Option<Vec<u8>>,
}

impl Query {
    /// Construct a new empty Query.
    ///
    /// ```
    /// # use google_cloud::datastore::Query;
    /// let query = Query::new("users");
    /// ```
    pub fn new(kind: impl Into<String>) -> Query {
        Query {
            kind: kind.into(),
            eventual: false,
            keys_only: false,
            offset: 0,
            limit: None,
            namespace: None,
            projections: Vec::new(),
            distinct_on: Vec::new(),
            ordering: Vec::new(),
            filters: Vec::new(),
            cursor: None,
        }
    }

    /// Ask to accept eventually consistent results.
    /// It only has an effect on ancestor queries.
    ///
    /// ```
    /// # use google_cloud::datastore::Query;
    /// let query = Query::new("users")
    ///     .eventually_consistent();
    /// ```
    pub fn eventually_consistent(mut self) -> Query {
        self.eventual = true;
        self
    }

    /// Ask to yield only yield keys, without the entity values.
    /// It has no effects on projected queries.
    ///
    /// ```
    /// # use google_cloud::datastore::Query;
    /// let query = Query::new("users").keys_only();
    /// ```
    pub fn keys_only(mut self) -> Query {
        self.keys_only = true;
        self
    }

    /// Skip any number of keys before returning results.
    ///
    /// ```
    /// # use google_cloud::datastore::Query;
    /// let query = Query::new("users").offset(14);
    /// ```
    pub fn offset(mut self, offset: i32) -> Query {
        self.offset = offset;
        self
    }

    /// Limit the number of results to send back.
    ///
    /// ```
    /// # use google_cloud::datastore::Query;
    /// let query = Query::new("users").limit(25);
    /// ```
    pub fn limit(mut self, limit: i32) -> Query {
        self.limit = Some(limit);
        self
    }

    /// Appends an ancestor filter to the query.
    ///
    /// ```
    /// # use google_cloud::datastore::Query;
    /// use google_cloud::datastore::Key;
    ///
    /// let key = Key::new("dev").id(10);
    /// let query = Query::new("users").ancestor(key);
    /// ```
    pub fn ancestor(mut self, key: Key) -> Query {
        self.filters.push(Filter::HasAncestor(key.into_value()));
        self
    }

    /// Associates the query with a namespace.
    ///
    /// ```
    /// # use google_cloud::datastore::Query;
    /// let query = Query::new("users").namespace("dev");
    /// ```
    pub fn namespace(mut self, namespace: impl Into<String>) -> Query {
        self.namespace = Some(namespace.into());
        self
    }

    /// Ask to only yield the given fields.
    ///
    /// ```
    /// # use google_cloud::datastore::Query;
    /// let fields: Vec<String> = vec![
    ///     "firstname".into(),
    ///     "lastname".into(),
    ///     "age".into()
    /// ];
    /// let query = Query::new("users").project(fields);
    /// ```
    pub fn project<T, I>(mut self, projections: I) -> Query
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.projections.clear();
        self.projections
            .extend(projections.into_iter().map(Into::into));
        self
    }

    /// Ask to yield de-duplicated results.
    ///
    /// ```
    /// # use google_cloud::datastore::Query;
    /// let fields: Vec<String> = vec!["email".into()];
    /// let query = Query::new("users").distinct_on(fields);
    /// ```
    pub fn distinct_on<T, I>(mut self, fields: I) -> Query
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.distinct_on.clear();
        self.distinct_on.extend(fields.into_iter().map(Into::into));
        self
    }

    /// Filter results based on their fields.
    /// Multiple filters are combined with an 'AND'.
    ///
    /// ```
    /// # use google_cloud::datastore::Query;
    /// use google_cloud::datastore::{Filter, Value, IntoValue};
    ///
    /// let query = Query::new("users")
    ///     .filter(Filter::GreaterThan("age".into(), 10.into_value()))
    ///     .filter(Filter::Equal("firstname".into(), "john".into_value()));
    /// ```
    pub fn filter(mut self, filter: Filter) -> Query {
        self.filters.push(filter);
        self
    }

    /// Order results based on some of their fields.
    /// Multiple orderings are applied in the order they are added.
    ///
    /// ```
    /// # use google_cloud::datastore::Query;
    /// use google_cloud::datastore::Order;
    ///
    /// let query = Query::new("users")
    ///     .order(Order::Asc("age".into()))
    ///     .order(Order::Desc("firstname".into()));
    /// ```
    pub fn order(mut self, order: Order) -> Query {
        self.ordering.push(order);
        self
    }

    /// We indicate by which entity the search begins, with this we can 
    /// implement a pagination system
    /// 
    /// ```
    /// let query = Query::new("users")
    ///     .cursor(cursor);
    /// ```
    /// 
    pub fn cursor(mut self, cursor: Vec<u8>) -> Query {
        self.cursor = Some(cursor);
        self
    }
}
