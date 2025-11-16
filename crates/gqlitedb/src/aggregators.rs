mod arithmetic;
mod containers;
mod count;
mod stats;

use std::{fmt::Debug, sync::Arc};

use crate::prelude::*;
use compiler::expression_analyser::ExpressionType;

pub(crate) trait AggregatorState: Debug
{
  fn next(&mut self, expression: value::Value) -> Result<()>;
  fn finalise(self: Box<Self>) -> Result<value::Value>;
}

pub(crate) trait AggregatorTrait: Debug + Sync + Send
{
  fn create(&self, arguments: Vec<value::Value>) -> Result<Box<dyn AggregatorState>>;
  fn validate_arguments(&self, arguments: Vec<ExpressionType>) -> Result<ExpressionType>;
}

pub(crate) type Aggregator = Arc<Box<dyn AggregatorTrait>>;

macro_rules! declare_aggregator {
  ($function_name: ident, $type_name: ident, $state_type_name: tt, (  $( $arg_type: ty $(,)? )* ) -> $ret_type: ty) => {
    #[derive(Debug)]
    pub(super) struct $type_name {}
    impl $type_name
    {
      pub(crate) fn create() -> (String, crate::aggregators::Aggregator)
      {
        (
          stringify!($function_name).to_string(),
          std::sync::Arc::new(Box::new(Self {})),
        )
      }
    }
    impl crate::aggregators::AggregatorTrait for $type_name
    {
      #[allow(unused_variables)]
      fn create(&self, arguments: Vec<crate::value::Value>) -> Result<Box<dyn AggregatorState>>
      {
        Ok(Box::new($crate::functions::make_function_call!($function_name, $state_type_name::new, arguments, $( $arg_type,)*)?))
      }
      #[allow(unused_variables)]
      fn validate_arguments(
        &self,
        arguments: Vec<$crate::compiler::expression_analyser::ExpressionType>,
      ) -> Result<$crate::compiler::expression_analyser::ExpressionType>
      {
        use crate::functions::FunctionTypeTrait;
        // TODO
        Ok(<$ret_type>::result_type())
      }

    }
  };
}

pub(crate) use declare_aggregator;

pub(crate) fn init_aggregators() -> std::collections::HashMap<String, Aggregator>
{
  [
    count::Count::create(),
    arithmetic::Sum::create(),
    containers::Collect::create(),
    stats::Avg::create(),
    stats::Min::create(),
    stats::Max::create(),
  ]
  .into()
}
