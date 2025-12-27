//! Modal field identifiers and navigation logic

/// Field identifiers for Connection modal
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionModalField {
    Name,
    Host,
    Port,
    User,
    Password,
    Database,
    ButtonOk,
    ButtonCancel,
}

impl ConnectionModalField {
    pub fn next(self) -> Self {
        match self {
            ConnectionModalField::Name => ConnectionModalField::Host,
            ConnectionModalField::Host => ConnectionModalField::Port,
            ConnectionModalField::Port => ConnectionModalField::User,
            ConnectionModalField::User => ConnectionModalField::Password,
            ConnectionModalField::Password => ConnectionModalField::Database,
            ConnectionModalField::Database => ConnectionModalField::ButtonOk,
            ConnectionModalField::ButtonOk => ConnectionModalField::ButtonCancel,
            ConnectionModalField::ButtonCancel => ConnectionModalField::Name,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ConnectionModalField::Name => ConnectionModalField::ButtonCancel,
            ConnectionModalField::Host => ConnectionModalField::Name,
            ConnectionModalField::Port => ConnectionModalField::Host,
            ConnectionModalField::User => ConnectionModalField::Port,
            ConnectionModalField::Password => ConnectionModalField::User,
            ConnectionModalField::Database => ConnectionModalField::Password,
            ConnectionModalField::ButtonOk => ConnectionModalField::Database,
            ConnectionModalField::ButtonCancel => ConnectionModalField::ButtonOk,
        }
    }
}

/// Field identifiers for Project modal
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectModalField {
    Name,
    ButtonOk,
    ButtonCancel,
}

impl ProjectModalField {
    pub fn next(self) -> Self {
        match self {
            ProjectModalField::Name => ProjectModalField::ButtonOk,
            ProjectModalField::ButtonOk => ProjectModalField::ButtonCancel,
            ProjectModalField::ButtonCancel => ProjectModalField::Name,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ProjectModalField::Name => ProjectModalField::ButtonCancel,
            ProjectModalField::ButtonOk => ProjectModalField::Name,
            ProjectModalField::ButtonCancel => ProjectModalField::ButtonOk,
        }
    }
}

/// Field identifiers for delete confirmation modal
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfirmModalField {
    ButtonOk,
    ButtonCancel,
}

impl ConfirmModalField {
    pub fn next(self) -> Self {
        match self {
            ConfirmModalField::ButtonOk => ConfirmModalField::ButtonCancel,
            ConfirmModalField::ButtonCancel => ConfirmModalField::ButtonOk,
        }
    }

    pub fn prev(self) -> Self {
        self.next()
    }
}
