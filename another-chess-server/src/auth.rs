/// A match-level unique identifier used by various components in the server.
pub type UserId = String;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    /// A match-level unique identifier used by various components in the server.
    pub id: UserId,

    /// A human-readable name chosen by the user.
    pub name: String,
}
