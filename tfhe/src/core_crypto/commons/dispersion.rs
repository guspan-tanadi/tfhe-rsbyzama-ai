//! Module containing noise distribution primitives.
//!
//! When dealing with noise, we tend to use different representation for the same value. In
//! general, the noise is specified by the standard deviation of a gaussian distribution, which
//! is of the form $\sigma = 2^p$, with $p$ a negative integer. Depending on the use case though,
//! we rely on different representations for this quantity:
//!
//! + $\sigma$ can be encoded in the [`StandardDev`] type.
//! + $p$ can be encoded in the [`LogStandardDev`] type.
//! + $\sigma^2$ can be encoded in the [`Variance`] type.
//!
//! In any of those cases, the corresponding type implements the `DispersionParameter` trait,
//! which makes if possible to use any of those representations generically when noise must be
//! defined.

use serde::{Deserialize, Serialize};
use tfhe_versionable::Versionize;

use crate::core_crypto::backward_compatibility::commons::dispersion::{
    StandardDevVersions, VarianceVersions,
};

/// A trait for types representing distribution parameters, for a given unsigned integer type.
//  Warning:
//  DispersionParameter type should ONLY wrap a single native type.
//  As long as Variance wraps a native type (f64) it is ok to derive it from Copy instead of
//  Clone because f64 is itself Copy and stored in register.
pub trait DispersionParameter: Copy {
    /// Return the standard deviation of the distribution, i.e. $\sigma = 2^p$.
    fn get_standard_dev(&self) -> StandardDev;
    /// Return the variance of the distribution, i.e. $\sigma^2 = 2^{2p}$.
    fn get_variance(&self) -> Variance;
    /// Return base 2 logarithm of the standard deviation of the distribution, i.e.
    /// $\log\_2(\sigma)=p$
    fn get_log_standard_dev(&self) -> LogStandardDev;
    /// For a `Uint` type representing $\mathbb{Z}/2^q\mathbb{Z}$, we return $2^{q-p}$.
    fn get_modular_standard_dev(&self, modulus: f64) -> ModularStandardDev;

    /// For a `Uint` type representing $\mathbb{Z}/2^q\mathbb{Z}$, we return $2^{2(q-p)}$.
    fn get_modular_variance(&self, modulus: f64) -> ModularVariance;

    /// For a `Uint` type representing $\mathbb{Z}/2^q\mathbb{Z}$, we return $q-p$.
    fn get_modular_log_standard_dev(&self, modulus: f64) -> ModularLogStandardDev;
}

/// A distribution parameter that uses the base-2 logarithm of the standard deviation as
/// representation.
///
/// # Example:
///
/// ```rust
/// use tfhe::core_crypto::commons::dispersion::{DispersionParameter, LogStandardDev};
/// let params = LogStandardDev::from_log_standard_dev(-25.);
/// assert_eq!(params.get_standard_dev().0, 2_f64.powf(-25.));
/// assert_eq!(params.get_log_standard_dev().0, -25.);
/// assert_eq!(params.get_variance().0, 2_f64.powf(-25.).powi(2));
/// assert_eq!(
///     params.get_modular_standard_dev(2_f64.powi(32)).value,
///     2_f64.powf(32. - 25.)
/// );
/// assert_eq!(
///     params.get_modular_log_standard_dev(2_f64.powi(32)).value,
///     32. - 25.
/// );
/// assert_eq!(
///     params.get_modular_variance(2_f64.powi(32)).value,
///     2_f64.powf(32. - 25.).powi(2)
/// );
///
/// let modular_params = LogStandardDev::from_modular_log_standard_dev(22., 32);
/// assert_eq!(modular_params.get_standard_dev().0, 2_f64.powf(-10.));
/// ```
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct LogStandardDev(pub f64);

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct ModularLogStandardDev {
    pub value: f64,
    pub modulus: f64,
}

impl LogStandardDev {
    pub fn from_log_standard_dev(log_std: f64) -> Self {
        Self(log_std)
    }

    pub fn from_modular_log_standard_dev(log_std: f64, log2_modulus: u32) -> Self {
        Self(log_std - log2_modulus as f64)
    }
}

impl DispersionParameter for LogStandardDev {
    fn get_standard_dev(&self) -> StandardDev {
        StandardDev(f64::powf(2., self.0))
    }
    fn get_variance(&self) -> Variance {
        Variance(f64::powf(2., self.0 * 2.))
    }
    fn get_log_standard_dev(&self) -> Self {
        Self(self.0)
    }
    fn get_modular_standard_dev(&self, modulus: f64) -> ModularStandardDev {
        ModularStandardDev {
            value: 2_f64.powf(self.0) * modulus,
            modulus,
        }
    }
    fn get_modular_variance(&self, modulus: f64) -> ModularVariance {
        let std_dev = 2_f64.powf(self.0) * modulus;

        ModularVariance {
            value: std_dev * std_dev,
            modulus,
        }
    }
    fn get_modular_log_standard_dev(&self, modulus: f64) -> ModularLogStandardDev {
        ModularLogStandardDev {
            value: modulus.log2() + self.0,
            modulus,
        }
    }
}

/// A distribution parameter that uses the standard deviation as representation.
///
/// # Example:
///
/// ```rust
/// use tfhe::core_crypto::commons::dispersion::{DispersionParameter, StandardDev};
/// let params = StandardDev::from_standard_dev(2_f64.powf(-25.));
/// assert_eq!(params.get_standard_dev().0, 2_f64.powf(-25.));
/// assert_eq!(params.get_log_standard_dev().0, -25.);
/// assert_eq!(params.get_variance().0, 2_f64.powf(-25.).powi(2));
/// assert_eq!(
///     params.get_modular_standard_dev(2_f64.powi(32)).value,
///     2_f64.powf(32. - 25.)
/// );
/// assert_eq!(
///     params.get_modular_log_standard_dev(2_f64.powi(32)).value,
///     32. - 25.
/// );
/// assert_eq!(
///     params.get_modular_variance(2_f64.powi(32)).value,
///     2_f64.powf(32. - 25.).powi(2)
/// );
/// ```
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Versionize)]
#[versionize(StandardDevVersions)]
pub struct StandardDev(pub f64);

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct ModularStandardDev {
    pub value: f64,
    pub modulus: f64,
}

impl StandardDev {
    pub fn from_standard_dev(std: f64) -> Self {
        Self(std)
    }

    pub fn from_modular_standard_dev(std: f64, log2_modulus: u32) -> Self {
        Self(std / 2_f64.powf(log2_modulus as f64))
    }
}

impl DispersionParameter for StandardDev {
    fn get_standard_dev(&self) -> Self {
        Self(self.0)
    }
    fn get_variance(&self) -> Variance {
        Variance(self.0.powi(2))
    }
    fn get_log_standard_dev(&self) -> LogStandardDev {
        LogStandardDev(self.0.log2())
    }
    fn get_modular_standard_dev(&self, modulus: f64) -> ModularStandardDev {
        ModularStandardDev {
            value: self.0 * modulus,
            modulus,
        }
    }
    fn get_modular_variance(&self, modulus: f64) -> ModularVariance {
        let std_dev = self.0 * modulus;

        ModularVariance {
            value: std_dev * std_dev,
            modulus,
        }
    }
    fn get_modular_log_standard_dev(&self, modulus: f64) -> ModularLogStandardDev {
        ModularLogStandardDev {
            value: modulus.log2() + self.0.log2(),
            modulus,
        }
    }
}

/// A distribution parameter that uses the variance as representation
///
/// # Example:
///
/// ```rust
/// use tfhe::core_crypto::commons::dispersion::{DispersionParameter, Variance};
/// let params = Variance::from_variance(2_f64.powi(-50));
/// assert_eq!(params.get_standard_dev().0, 2_f64.powf(-25.));
/// assert_eq!(params.get_log_standard_dev().0, -25.);
/// assert_eq!(params.get_variance().0, 2_f64.powf(-25.).powi(2));
/// assert_eq!(
///     params.get_modular_standard_dev(2_f64.powi(32)).value,
///     2_f64.powf(32. - 25.)
/// );
/// assert_eq!(
///     params.get_modular_log_standard_dev(2_f64.powi(32)).value,
///     32. - 25.
/// );
/// assert_eq!(
///     params.get_modular_variance(2_f64.powi(32)).value,
///     2_f64.powf(32. - 25.).powi(2)
/// );
/// ```
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Versionize)]
#[versionize(VarianceVersions)]
pub struct Variance(pub f64);

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct ModularVariance {
    pub value: f64,
    pub modulus: f64,
}

impl Variance {
    pub fn from_variance(var: f64) -> Self {
        Self(var)
    }

    pub fn from_modular_variance(var: f64, modulus: f64) -> Self {
        Self(var / (modulus * modulus))
    }
}

impl DispersionParameter for Variance {
    fn get_standard_dev(&self) -> StandardDev {
        StandardDev(self.0.sqrt())
    }
    fn get_variance(&self) -> Self {
        Self(self.0)
    }
    fn get_log_standard_dev(&self) -> LogStandardDev {
        LogStandardDev(self.0.sqrt().log2())
    }
    fn get_modular_standard_dev(&self, modulus: f64) -> ModularStandardDev {
        ModularStandardDev {
            value: self.0.sqrt() * modulus,
            modulus,
        }
    }
    fn get_modular_variance(&self, modulus: f64) -> ModularVariance {
        ModularVariance {
            value: self.0 * modulus * modulus,
            modulus,
        }
    }
    fn get_modular_log_standard_dev(&self, modulus: f64) -> ModularLogStandardDev {
        ModularLogStandardDev {
            value: modulus.log2() + self.0.sqrt().log2(),
            modulus,
        }
    }
}
