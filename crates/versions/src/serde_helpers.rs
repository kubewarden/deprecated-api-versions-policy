pub mod semver_serde {
    use semver::Version;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(version: &Version, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&version.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        let version_str = if s.starts_with('v') {
            s.strip_prefix('v').unwrap().to_string()
        } else {
            s
        };
        Version::parse(&version_str).map_err(serde::de::Error::custom)
    }
}

pub mod option_semver_serde {
    use semver::Version;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(version: &Option<Version>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(ref v) = *version {
            return s.serialize_str(&v.to_string());
        }
        s.serialize_none()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Version>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        if let Some(s) = s {
            if s.is_empty() {
                return Ok(None);
            }

            let version_str = if s.starts_with('v') {
                s.strip_prefix('v').unwrap().to_string()
            } else {
                s
            };

            return Ok(Some(
                Version::parse(&version_str).map_err(serde::de::Error::custom)?,
            ));
        }

        Ok(None)
    }
}
