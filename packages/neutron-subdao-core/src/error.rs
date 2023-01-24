use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ContractError {
    #[error("config name cannot be empty.")]
    NameIsEmpty {},

    #[error("config description cannot be empty.")]
    DescriptionIsEmpty {},

    #[error("config DAO URI cannot be empty.")]
    DaoUriIsEmpty {},
}
