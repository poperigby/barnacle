use barnacle_l10n::Translations;
use sea_orm::{prelude::*, sea_query::ValueTypeErr};

use crate::integrations::openmw;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, DeriveValueType, Hash, strum::Display)]
#[sea_orm(
    value_type = "String",
    from_str = "Kind::from_db_key",
    to_str = "Kind::as_db_key"
)]
pub enum Kind {
    OpenMW(openmw::ItemKind),
}

impl Kind {
    pub fn name(&self, translations: &Translations) -> String {
        match self {
            Kind::OpenMW(kind) => kind.name(translations),
        }
    }

    pub fn supports_reordering(&self) -> bool {
        match self {
            Kind::OpenMW(kind) => kind.supports_reordering(),
        }
    }

    pub fn supports_auto_sort(&self) -> bool {
        match self {
            Kind::OpenMW(kind) => kind.supports_auto_sort(),
        }
    }

    pub(crate) fn as_db_key(&self) -> String {
        match self {
            Kind::OpenMW(kind) => format!("openmw.{}", kind.key()),
        }
    }

    pub(crate) fn from_db_key(s: &str) -> Result<Self, ValueTypeErr> {
        let (deployer, kind) = s.split_once(".").unwrap();

        match deployer {
            "openmw" => kind.parse().map(Self::OpenMW).map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }
}
