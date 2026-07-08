//! Macro generating the stable database-string conversions for model enums.

/// Implement `ALL`, `as_db_str`, and `from_db_str` for a fieldless enum.
///
/// The database string is exactly the variant identifier, so renaming a
/// variant changes the on-disk format and requires a migration. Golden tests
/// in this module lock the current strings.
macro_rules! impl_db_str {
    ($ty:ident { $($variant:ident),+ $(,)? }, fallback = $fb:ident) => {
        impl $ty {
            /// All variants, in declaration (discriminant) order.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// Stable string used for database persistence.
            /// Do not change without a migration.
            #[must_use]
            pub const fn as_db_str(self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant)),+
                }
            }

            /// Parse the database string; unknown values fall back to
            #[doc = concat!("`", stringify!($ty), "::", stringify!($fb), "`.")]
            #[must_use]
            pub fn from_db_str(s: &str) -> Self {
                Self::ALL
                    .iter()
                    .copied()
                    .find(|v| v.as_db_str() == s)
                    .unwrap_or(Self::$fb)
            }
        }
    };
}

pub(crate) use impl_db_str;

#[cfg(test)]
mod tests {
    use crate::models::{AggregationTemporality, MetricType, SeverityLevel, SpanKind};

    #[test]
    fn test_db_str_golden_values() {
        // On-disk format lock: existing databases store these exact strings.
        assert_eq!(SpanKind::Server.as_db_str(), "Server");
        assert_eq!(SpanKind::Unspecified.as_db_str(), "Unspecified");
        assert_eq!(SeverityLevel::Info.as_db_str(), "Info");
        assert_eq!(SeverityLevel::Fatal4.as_db_str(), "Fatal4");
        assert_eq!(MetricType::Histogram.as_db_str(), "Histogram");
        assert_eq!(
            MetricType::ExponentialHistogram.as_db_str(),
            "ExponentialHistogram"
        );
        assert_eq!(AggregationTemporality::Cumulative.as_db_str(), "Cumulative");
    }

    #[test]
    fn test_db_str_roundtrip_all_variants() {
        for kind in SpanKind::ALL {
            assert_eq!(SpanKind::from_db_str(kind.as_db_str()), *kind);
        }
        for level in SeverityLevel::ALL {
            assert_eq!(SeverityLevel::from_db_str(level.as_db_str()), *level);
        }
        for metric_type in MetricType::ALL {
            assert_eq!(
                MetricType::from_db_str(metric_type.as_db_str()),
                *metric_type
            );
        }
        for temporality in AggregationTemporality::ALL {
            assert_eq!(
                AggregationTemporality::from_db_str(temporality.as_db_str()),
                *temporality
            );
        }
    }

    #[test]
    fn test_from_db_str_unknown_falls_back() {
        assert_eq!(SpanKind::from_db_str("bogus"), SpanKind::Unspecified);
        assert_eq!(
            SeverityLevel::from_db_str("bogus"),
            SeverityLevel::Unspecified
        );
        assert_eq!(MetricType::from_db_str("bogus"), MetricType::Gauge);
        assert_eq!(
            AggregationTemporality::from_db_str("bogus"),
            AggregationTemporality::Unspecified
        );
    }
}
