//! This library provides an simple, lightweight, and efficient (though not heavily optimized) implementation of the Black-Scholes-Merton model for pricing European options.
//!
//! Provides methods for pricing options, calculating implied volatility, and calculating the first, second, and third order Greeks.
//!
//! ### Example:
//! ```
//! ```
//!
//! Criterion benchmark can be ran by running:
//! ```bash
//! cargo bench
//! ```
//!
//! See the [Github Repo](https://github.com/hayden4r4/blackscholes-rust/tree/master) for full source code.  Other implementations such as a [npm WASM package](https://www.npmjs.com/package/@haydenr4/blackscholes_wasm) and a [python module](https://pypi.org/project/blackscholes/) are also available.

mod lets_be_rational;

use statrs::distribution::{ContinuousCDF, Normal};

pub const SQRT_2PI: f64 = 2.5066282;
pub const DAYS_PER_YEAR: f64 = 365.25;
pub use std::f64::consts::PI;

pub const A: f64 = 4.62627532e-01;
pub const B: f64 = -1.16851917e-02;
pub const C: f64 = 9.63541838e-04;
pub const D: f64 = 7.53502261e-05;
pub const _E: f64 = 1.42451646e-05;
pub const F: f64 = -2.10237683e-05;

fn calculate_npdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / SQRT_2PI
}

/// The inputs to the Black-Scholes-Merton model.
#[derive(Debug, Clone)]
pub struct OptionInputs {
    /// The type of the option (call or put)
    pub is_call: bool,

    /// Stock price
    pub s: f64,

    /// Strike price
    pub k: f64,

    /// Risk-free rate
    pub r: f64,

    /// Dividend yield
    pub q: f64,

    /// Time to maturity in years
    pub t: f64,

    /// Implied vol
    pub implied_vol: f64,

    /// Option price
    pub price: f64,

    /// Cache intermediate results to speed up subsequent calculations.
    d1: f64,
    d2: f64,
    nd1: f64,
    nd2: f64,
    nprimed1: f64,
    nprimed2: f64,
}

/// Methods for calculating the price, greeks, and implied volatility of an option.
impl OptionInputs {
    pub fn new(is_call: bool, s: f64, k: f64, r: f64, q: f64, t: f64) -> Self {
        Self {
            is_call,
            s,
            k,
            r,
            q,
            t,
            implied_vol: f64::NAN,
            price: f64::NAN,
            d1: f64::NAN,
            d2: f64::NAN,
            nd1: f64::NAN,
            nd2: f64::NAN,
            nprimed1: f64::NAN,
            nprimed2: f64::NAN,
        }
    }

    pub fn with_implied_vol(mut self, implied_vol: f64) -> Self {
        self.implied_vol = implied_vol;

        // Calculate d1, d2
        let numerator =
            (self.s / self.k).ln() + (self.r - self.q + implied_vol.powi(2) / 2.0) * self.t;

        let denominator = implied_vol * self.t.sqrt();
        self.d1 = numerator / denominator;
        self.d2 = self.d1 - denominator;

        // Then nd1, nd2
        let n = Normal::new(0.0, 1.0).unwrap();
        self.nd1 = n.cdf(self.sign() * self.d1);
        self.nd2 = n.cdf(self.sign() * self.d2);

        // Then nprimed1, nprimed2
        self.nprimed1 = calculate_npdf(self.d1);
        self.nprimed2 = calculate_npdf(self.d2);

        if !self.price.is_finite() {
            // let's be rational wants the forward price, not the spot price.
            let forward = self.s * ((self.r - self.q) * self.t).exp();

            // convert the option type into \theta
            // price using `black`
            let undiscounted_price =
                lets_be_rational::black(forward, self.k, implied_vol, self.t, self.sign());

            // discount the price
            self.price = undiscounted_price * self.rate_discount();
        }

        self
    }

    pub fn with_price(mut self, p: f64) -> Self {
        // "let's be rational" works with the forward and undiscounted option price, so remove the discount
        let rate_inv_discount = (self.r * self.t).exp();
        let p = p * rate_inv_discount;

        // compute the forward price
        let f = self.s * rate_inv_discount;

        // The Black-Scholes-Merton formula takes into account dividend yield by setting S = S * e^{-qt}, do this here with the forward
        let f = f * self.dividend_discount();

        // convert the option type into \theta
        let implied_vol = lets_be_rational::implied_volatility_from_a_transformed_rational_guess(
            p,
            f,
            self.k,
            self.t,
            self.sign(),
        );

        if implied_vol > 0.0 {
            self.price = p;
            self.with_implied_vol(implied_vol)
        } else {
            self
        }
    }

    #[inline(always)]
    pub fn sign(&self) -> f64 {
        if self.is_call {
            1.0
        } else {
            -1.0
        }
    }

    #[inline(always)]
    pub fn dividend_discount(&self) -> f64 {
        (-self.q * self.t).exp()
    }

    #[inline(always)]
    pub fn rate_discount(&self) -> f64 {
        (-self.r * self.t).exp()
    }

    pub fn implied_vol(&self) -> f64 {
        self.implied_vol
    }

    pub fn price(&self) -> f64 {
        self.price
    }

    pub fn delta(&self) -> f64 {
        self.sign() * self.nd1 * self.dividend_discount()
    }

    pub fn gamma(&self) -> f64 {
        self.dividend_discount() * self.nprimed1 / (self.s * self.implied_vol * self.t.sqrt())
    }

    pub fn theta(&self) -> f64 {
        let dividend_discount = self.dividend_discount();

        (-(self.s * self.implied_vol * dividend_discount * self.nprimed1 / (2.0 * self.t.sqrt()))
            - self.sign() * self.r * self.k * self.rate_discount() * self.nd2
            + self.sign() * self.q * self.s * dividend_discount * self.nd1)
            / DAYS_PER_YEAR
    }

    pub fn vega(&self) -> f64 {
        0.01 * self.s * self.dividend_discount() * self.t.sqrt() * self.nprimed1
    }

    pub fn rho(&self) -> f64 {
        self.sign() * 0.01 * self.k * self.t * self.rate_discount() * self.nd2
    }

    pub fn epsilon(&self) -> f64 {
        -self.sign() * self.s * self.t * self.dividend_discount() * self.nd1
    }

    pub fn lambda(&self) -> f64 {
        self.delta() * self.s / self.price
    }

    pub fn vanna(&self) -> f64 {
        self.d2 * self.dividend_discount() * self.nprimed1 * -0.01 / self.implied_vol
    }

    pub fn charm(&self) -> f64 {
        let dividend_discount = self.dividend_discount();

        self.sign() * self.q * dividend_discount * self.nd1
            - dividend_discount
                * self.nprimed1
                * (2.0 * (self.r - self.q) * self.t - self.d2 * self.implied_vol * self.t.sqrt())
                / (2.0 * self.t * self.implied_vol * self.t.sqrt())
    }

    pub fn veta(&self) -> f64 {
        -self.s
            * self.dividend_discount()
            * self.nprimed1
            * self.t.sqrt()
            * (self.q + ((self.r - self.q) * self.d1) / (self.implied_vol * self.t.sqrt())
                - ((1.0 + self.d1 * self.d2) / (2.0 * self.t)))
    }

    pub fn vomma(&self) -> f64 {
        self.vega() * self.d1 * self.d2 / self.implied_vol
    }

    pub fn speed(&self) -> f64 {
        -self.gamma() / self.s * (self.d1 / (self.implied_vol * self.t.sqrt()) + 1.0)
    }

    pub fn zomma(&self) -> f64 {
        self.gamma() * ((self.d1 * self.d2 - 1.0) / self.implied_vol)
    }

    pub fn color(&self) -> f64 {
        -self.dividend_discount()
            * (self.nprimed1 / (2.0 * self.s * self.t * self.implied_vol * self.t.sqrt()))
            * (2.0 * self.q * self.t
                + 1.0
                + (2.0 * (self.r - self.q) * self.t - self.d2 * self.implied_vol * self.t.sqrt())
                    / (self.implied_vol * self.t.sqrt())
                    * self.d1)
    }

    pub fn ultima(&self) -> f64 {
        -self.vega() / (self.implied_vol * self.implied_vol)
            * (self.d1 * self.d2 * (1.0 - self.d1 * self.d2)
                + self.d1.powf(2.0)
                + self.d2.powf(2.0))
    }

    pub fn dual_delta(&self) -> f64 {
        -self.sign() * self.dividend_discount() * self.nd2
    }

    pub fn dual_gamma(&self) -> f64 {
        self.dividend_discount() * (self.nprimed2 / (self.k * self.implied_vol * self.t.sqrt()))
    }
}
