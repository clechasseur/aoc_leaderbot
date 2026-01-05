//! DynamoDB table configuration parameters.

use aws_sdk_dynamodb::operation::create_table::builders::CreateTableFluentBuilder;
use aws_sdk_dynamodb::types::{BillingMode, OnDemandThroughput, ProvisionedThroughput};

/// Default value of the [`read_capacity_units`] property when creating
/// a provisioned [`BillingModeConfig`].
///
/// [`read_capacity_units`]: ProvisionedThroughput::read_capacity_units
pub const DEFAULT_READ_CAPACITY_UNITS: i64 = 5;

/// Default value of the [`write_capacity_units`] property when creating
/// a provisioned [`BillingModeConfig`].
///
/// [`write_capacity_units`]: ProvisionedThroughput::write_capacity_units
pub const DEFAULT_WRITE_CAPACITY_UNITS: i64 = 5;

/// Configuration parameters for DynamoDB table used by [`DynamoDbStorage`].
///
/// [`DynamoDbStorage`]: crate::leaderbot::storage::aws::dynamodb::DynamoDbStorage
#[derive(Debug, Default, Clone)]
pub struct TableConfig {
    /// Config for the table's [billing mode].
    ///
    /// If not specified, the [default configuration] will be used.
    ///
    /// [billing mode]: aws_sdk_dynamodb::operation::create_table::builders::CreateTableFluentBuilder::billing_mode
    /// [default configuration]: BillingModeConfig
    pub billing_mode: Option<BillingModeConfig>,
}

/// Billing mode configuration for a DynamoDB table.
///
/// The default configuration uses [provisioned capacity mode], with a default of 5
/// [read and write capacity units].
///
/// [provisioned capacity mode]: https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/provisioned-capacity-mode.html
/// [read and write capacity units]: https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/provisioned-capacity-mode.html#read-write-capacity-units
#[derive(Debug, Clone)]
pub enum BillingModeConfig {
    /// Configure table in [on-demand capacity mode], with optional throughput limits.
    ///
    /// [on-demand capacity mode]: https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/on-demand-capacity-mode.html
    PayPerRequest(Option<OnDemandThroughput>),

    /// Configure table in [provisioned capacity mode], using the given provisioned capacity units.
    ///
    /// [provisioned capacity mode]: https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/provisioned-capacity-mode.html
    Provisioned(ProvisionedThroughput),
}

impl BillingModeConfig {
    /// Creates a [`PayPerRequest`] billing mode config with unconstrained throughput.
    ///
    /// [`PayPerRequest`]: Self::PayPerRequest
    pub fn unconstrained_pay_per_request() -> Self {
        Self::PayPerRequest(None)
    }

    /// Creates a [`PayPerRequest`] billing mode config with optional max number
    /// of [read and write request units].
    ///
    /// [`PayPerRequest`]: Self::PayPerRequest
    /// [read and write request units]: https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/on-demand-capacity-mode.html#read-write-request-units
    pub fn pay_per_request(
        max_read_request_units: Option<i64>,
        max_write_request_units: Option<i64>,
    ) -> Self {
        Self::PayPerRequest(match (max_read_request_units, max_write_request_units) {
            (None, None) => None,
            (max_read_request_units, max_write_request_units) => Some(
                OnDemandThroughput::builder()
                    .set_max_read_request_units(max_read_request_units)
                    .set_max_write_request_units(max_write_request_units)
                    .build(),
            ),
        })
    }

    /// Creates a [`Provisioned`] billing mode config with the given
    /// [read and write capacity units].
    ///
    /// [`Provisioned`]: Self::Provisioned
    /// [read and write capacity units]: https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/provisioned-capacity-mode.html#read-write-capacity-units
    pub fn provisioned(read_capacity_units: i64, write_capacity_units: i64) -> Self {
        Self::Provisioned(
            ProvisionedThroughput::builder()
                .read_capacity_units(read_capacity_units)
                .write_capacity_units(write_capacity_units)
                .build()
                .expect("all parameters should have been provided"),
        )
    }
}

impl Default for BillingModeConfig {
    fn default() -> Self {
        Self::provisioned(DEFAULT_READ_CAPACITY_UNITS, DEFAULT_WRITE_CAPACITY_UNITS)
    }
}

pub(crate) trait CreateTableBuilderExt {
    fn table_config(self, config: Option<TableConfig>) -> Self;
}

impl CreateTableBuilderExt for CreateTableFluentBuilder {
    fn table_config(self, config: Option<TableConfig>) -> Self {
        match config.and_then(|c| c.billing_mode).unwrap_or_default() {
            BillingModeConfig::PayPerRequest(on_demand_throughput) => self
                .billing_mode(BillingMode::PayPerRequest)
                .set_on_demand_throughput(on_demand_throughput),
            BillingModeConfig::Provisioned(provisioned_throughput) => self
                .billing_mode(BillingMode::Provisioned)
                .provisioned_throughput(provisioned_throughput),
        }
    }
}
