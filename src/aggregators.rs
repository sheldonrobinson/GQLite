mod count;

use std::fmt::Debug;

use crate::interpreter::expression_analyser::ExpressionType;
use crate::{graph, value_table, Result};

pub(crate) trait AggregatorState: Debug
{
  fn next(&mut self, expression: graph::Value) -> Result<()>;
  fn finalise(self) -> Result<graph::Value>;
}

pub(crate) trait AggregatorTrait: Debug
{
  fn create(&self, arguments: Vec<graph::Value>) -> Result<Box<dyn AggregatorState>>;
  fn validate_arguments(&self, arguments: Vec<ExpressionType>) -> Result<ExpressionType>;
}

pub(crate) type Aggregator = std::rc::Rc<Box<dyn AggregatorTrait>>;

macro_rules! declare_aggregator {
  ($function_name: ident, $type_name: tt, $state_type_name: tt, (  $( $arg_type: ty $(,)? )* ) -> $ret_type: ident) => {
    #[derive(Debug)]
    pub(super) struct $type_name {}
    impl $type_name
    {
      pub(crate) fn new() -> (String, crate::aggregators::Aggregator)
      {
        (
          stringify!($function_name).to_string(),
          std::rc::Rc::new(Box::new(Self {})),
        )
      }
    }
    impl crate::aggregators::AggregatorTrait for $type_name
    {
      fn create(&self, arguments: Vec<crate::graph::Value>) -> Result<Box<dyn AggregatorState>>
      {
        Ok(Box::new($crate::functions::make_function_call!($state_type_name::new, arguments, $( $arg_type,)*)?))
      }
      fn validate_arguments(
        &self,
        arguments: Vec<crate::interpreter::expression_analyser::ExpressionType>,
      ) -> Result<crate::interpreter::expression_analyser::ExpressionType>
      {
        use crate::functions::FunctionTypeTrait;
        // TODO
        Ok($ret_type::result_type())
      }

    }
  };
}

pub(crate) use declare_aggregator;

pub(crate) fn init_aggregators() -> std::collections::HashMap<String, Aggregator>
{
  [count::Count::new()].into()
}
