/// Represents an expression text. Example:
///
///     title: "User account presence"
///     description: "Determines whether the request has a user account"
///     expression: "size(request.user) > 0"
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Expr {
    /// Textual representation of an expression in
    /// Common Expression Language syntax.
    ///
    /// The application context of the containing message determines which
    /// well-known feature set of CEL is supported.
    #[prost(string, tag = "1")]
    pub expression: std::string::String,
    /// An optional title for the expression, i.e. a short string describing
    /// its purpose. This can be used e.g. in UIs which allow to enter the
    /// expression.
    #[prost(string, tag = "2")]
    pub title: std::string::String,
    /// An optional description of the expression. This is a longer text which
    /// describes the expression, e.g. when hovered over it in a UI.
    #[prost(string, tag = "3")]
    pub description: std::string::String,
    /// An optional string indicating the location of the expression for error
    /// reporting, e.g. a file name and a position in the file.
    #[prost(string, tag = "4")]
    pub location: std::string::String,
}
