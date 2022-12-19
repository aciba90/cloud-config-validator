use serde::ser::SerializeStruct;
use serde::Serialize;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
    InvalidYaml(serde_yaml::Error),
}

impl Error {
    fn inner_error(&self) -> &serde_yaml::Error {
        match self {
            Self::InvalidYaml(e) => e,
        }
    }

    fn location(&self) -> Option<Location> {
        let inner_loc = self.inner_error().location();
        match inner_loc {
            None => {
                return None;
            }
            Some(inner_loc) => Some(Location::from_serde_location(inner_loc)),
        }
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("error", 1)?;

        let loc = &self.location();
        let loc = match loc {
            None => None,
            Some(location) => Some(Location {
                index: location.index(),
                line: location.line(),
                column: location.column(),
            }),
        };
        state.serialize_field("location", &loc)?;
        state.end()
    }
}

#[derive(Debug, Serialize)]
pub struct Location {
    index: usize,
    line: usize,
    column: usize,
}

impl Location {
    /// The byte index of the error
    pub fn index(&self) -> usize {
        self.index
    }

    /// The line of the error
    pub fn line(&self) -> usize {
        self.line
    }

    /// The column of the error
    pub fn column(&self) -> usize {
        self.column
    }

    #[doc(hidden)]
    fn from_serde_location(location: serde_yaml::Location) -> Self {
        Location {
            index: location.index(),
            line: location.line(),
            column: location.column(),
        }
    }
}
