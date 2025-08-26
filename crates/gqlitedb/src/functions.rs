use std::{collections::HashMap, fmt::Debug, sync::Arc};

mod containers;
mod edge;
mod math;
mod node;
mod path;
mod scalar;
mod string;
mod value;

pub(crate) type FResult<T> = std::result::Result<T, error::RunTimeError>;

use crate::prelude::*;
use ccutils::sync::ArcRwLock;
use compiler::expression_analyser::ExpressionType;

pub(crate) trait FunctionTypeTrait
{
  fn result_type() -> ExpressionType;
}

impl FunctionTypeTrait for crate::value::Value
{
  fn result_type() -> ExpressionType
  {
    ExpressionType::Variant
  }
}
impl FunctionTypeTrait for bool
{
  fn result_type() -> ExpressionType
  {
    ExpressionType::Boolean
  }
}

impl FunctionTypeTrait for String
{
  fn result_type() -> ExpressionType
  {
    ExpressionType::String
  }
}

impl FunctionTypeTrait for i64
{
  fn result_type() -> ExpressionType
  {
    ExpressionType::Integer
  }
}

impl FunctionTypeTrait for f64
{
  fn result_type() -> ExpressionType
  {
    ExpressionType::Float
  }
}

impl<T> FunctionTypeTrait for Vec<T>
{
  fn result_type() -> ExpressionType
  {
    ExpressionType::Array
  }
}

impl<T> FunctionTypeTrait for HashMap<String, T>
{
  fn result_type() -> ExpressionType
  {
    ExpressionType::Map
  }
}

impl FunctionTypeTrait for crate::value::ValueMap
{
  fn result_type() -> ExpressionType
  {
    ExpressionType::Map
  }
}

//  _____                 _   _           _____          _ _
// |  ___|   _ _ __   ___| |_(_) ___  _ _|_   _| __ __ _(_) |_
// | |_ | | | | '_ \ / __| __| |/ _ \| '_ \| || '__/ _` | | __|
// |  _|| |_| | | | | (__| |_| | (_) | | | | || | | (_| | | |_
// |_|   \__,_|_| |_|\___|\__|_|\___/|_| |_|_||_|  \__,_|_|\__|

pub(crate) trait FunctionTrait: Debug + Sync + Send
{
  fn call(&self, arguments: Vec<crate::value::Value>) -> Result<crate::value::Value>;
  fn validate_arguments(&self, arguments: Vec<ExpressionType>) -> Result<ExpressionType>;
  fn is_deterministic(&self) -> bool;
}

//  _____                 _   _
// |  ___|   _ _ __   ___| |_(_) ___  _ __
// | |_ | | | | '_ \ / __| __| |/ _ \| '_ \
// |  _|| |_| | | | | (__| |_| | (_) | | | |
// |_|   \__,_|_| |_|\___|\__|_|\___/|_| |_|

pub(crate) type Function = Arc<Box<dyn FunctionTrait>>;

//  __  __
// |  \/  | __ _ _ __   __ _  __ _  ___ _ __
// | |\/| |/ _` | '_ \ / _` |/ _` |/ _ \ '__|
// | |  | | (_| | | | | (_| | (_| |  __/ |
// |_|  |_|\__,_|_| |_|\__,_|\__, |\___|_|
//                           |___/

#[derive(Debug)]
struct ManagerInner
{
  functions: HashMap<String, Function>,
  aggregators: HashMap<String, aggregators::Aggregator>,
}

#[derive(Debug, Clone)]
pub(crate) struct Manager
{
  inner: ArcRwLock<ManagerInner>,
}

ccutils::assert_impl_all!(Manager: Sync, Send);

impl Manager
{
  pub(crate) fn new() -> Self
  {
    Self {
      inner: ManagerInner {
        functions: HashMap::from([
          containers::Head::create(),
          containers::Keys::create(),
          containers::Range::create(),
          containers::Size::create(),
          edge::Type::create(),
          math::Ceil::create(),
          math::Floor::create(),
          math::Rand::create(),
          node::Labels::create(),
          path::Length::create(),
          path::Nodes::create(),
          path::Edges::create(),
          scalar::Coalesce::create(),
          scalar::Properties::create(),
          scalar::ToInteger::create(),
          string::ToString::create(),
          value::HasLabel::create(),
          value::HasLabels::create(),
        ]),
        aggregators: aggregators::init_aggregators(),
      }
      .into(),
    }
  }
  pub(crate) fn get_function<E: error::GenericErrors>(&self, name: &str) -> Result<Function>
  {
    Ok(
      self
        .inner
        .read()?
        .functions
        .get(&name.to_lowercase())
        .ok_or_else(|| E::unknown_function(name).into())?
        .clone(),
    )
  }
  pub(crate) fn get_aggregator<E: error::GenericErrors>(
    &self,
    name: &str,
  ) -> Result<aggregators::Aggregator>
  {
    Ok(
      self
        .inner
        .read()?
        .aggregators
        .get(&name.to_lowercase())
        .ok_or_else(|| E::unknown_function(name).into())?
        .clone(),
    )
  }
  pub(crate) fn is_deterministic(&self, name: impl Into<String>) -> Result<bool>
  {
    let name = name.into();
    let fun = self.get_function::<crate::error::CompileTimeError>(&name);
    match fun
    {
      Ok(fun) => Ok(fun.is_deterministic()),
      Err(_) =>
      {
        self.get_aggregator::<crate::error::CompileTimeError>(&name)?;
        Ok(false)
      }
    }
  }
  pub(crate) fn is_aggregate(&self, name: &String) -> Result<bool>
  {
    Ok(self.inner.read()?.aggregators.contains_key(name))
  }

  pub(crate) fn validate_arguments(
    &self,
    name: impl Into<String>,
    arguments: Vec<ExpressionType>,
  ) -> Result<ExpressionType>
  {
    let name = name.into();
    let fun = self.get_function::<crate::error::CompileTimeError>(&name);
    match fun
    {
      Ok(fun) => fun.validate_arguments(arguments),
      Err(_) => self
        .get_aggregator::<crate::error::CompileTimeError>(&name)?
        .validate_arguments(arguments),
    }
  }
}

macro_rules! make_function_argument {
  ($function_name: ident, $arguments: ident, $index: expr, $arg_type: ty) => {
    $arguments[$index]
      .try_into_ref()
      .map_err(|_| RunTimeError::InvalidArgument {
        function_name: stringify!($function_name),
        index: $index,
        expected_type: stringify!($arg_type),
        value: format!("{:?}", $arguments[$index]),
      })?
  };
}

macro_rules! make_function_call {
  ($function_name: ident, $function: expr, $arguments: ident, $arg_type_0: ty, ) => {
    $function($crate::functions::make_function_argument!(
      $function_name,
      $arguments,
      0,
      $arg_type_0
    ))
  };
  ($function_name: ident, $function: expr, $arguments: ident, $arg_type_0: ty, $arg_type_1: ty, ) => {
    $function(
      $crate::functions::make_function_argument!($function_name, $arguments, 0, $arg_type_0),
      $crate::functions::make_function_argument!($function_name, $arguments, 1, $arg_type_1),
    )
  };
  ($function_name: ident, $function: expr, $arguments: ident, ) => {
    $function()
  };
}

macro_rules! count_arguments {
  ($count: expr, ) => {
    $count
  };
  ($count: expr, $arg_type_0: ty, $( $arg_type: ty , )*) => {
    $crate::functions::count_arguments!($count + 1, $( $arg_type, )*)
  };
  ($( $arg_type: ty $(,)? )* ) => {
    $crate::functions::count_arguments!(0, $( $arg_type, )*)
  };
}

macro_rules! count_patterns {
  ($count: expr, ) => {
    $count
  };
  ($count: expr, $arg_type_0: pat, $( $arg_pat: pat , )*) => {
    $crate::functions::count_patterns!($count + 1, $( $arg_pat, )*)
  };
  ($( $arg_pat: pat $(,)? )* ) => {
    $crate::functions::count_patterns!(0, $( $arg_pat, )*)
  };
}

#[rustfmt::skip]
macro_rules! default_validate_ {
  ($function_name: ident, $ret_type: ty) => {
    |_: Vec<$crate::compiler::expression_analyser::ExpressionType>|
      -> crate::Result<$crate::compiler::expression_analyser::ExpressionType>
    {
      // TODO
      use $crate::functions::FunctionTypeTrait;
      Ok(<$ret_type>::result_type())
    }
  };
}

#[rustfmt::skip]
macro_rules! validate_args_ {
  ($function_name: ident, $ret_type: ty, $( $expression_type: pat ),* ) => {
    |args: Vec<$crate::compiler::expression_analyser::ExpressionType>|
      -> crate::Result<$crate::compiler::expression_analyser::ExpressionType>
    {
      const ARG_COUNT: usize = $crate::functions::count_patterns!($( $expression_type,)*); 
      if args.len() != ARG_COUNT
      {
        Err(crate::error::CompileTimeError::InvalidNumberOfArguments {
          function_name: stringify!($function_name),
          got: args.len(),
          expected: ARG_COUNT
        })?;
      }
      let mut it = args.into_iter();
      $(
        match it.next().unwrap()
        {
          $expression_type | ExpressionType::Variant => 
          Ok(<$ret_type>::result_type()),
          _ => Err(crate::error::CompileTimeError::InvalidArgumentType.into())
        }
      )*
    }
  };
}

macro_rules! declare_function_ {
  ($function_name: ident, $type_name: ty, $f_name: ident (  $( $arg_type: ty $(,)? )* ) -> $ret_type: ty, $allow_null: expr, $validator: block ) => {
    impl $type_name
    {
      pub(super) fn create() -> (String, crate::functions::Function)
      {
        (
          stringify!($function_name).to_string(),
          std::sync::Arc::new(Box::new(Self {})),
        )
      }
    }
    impl crate::functions::FunctionTrait for $type_name
    {
      fn call(&self, arguments: Vec<crate::value::Value>) -> crate::Result<crate::value::Value>
      {
        const ARG_COUNT: usize = $crate::functions::count_arguments!($( $arg_type,)*);
        if arguments.len() == ARG_COUNT
        {
          if !$allow_null && ARG_COUNT > 0 && arguments.iter().all(|x| x.is_null())
          {
            return Ok(crate::value::Value::Null)
          }
          #[allow(unused_imports)]
          use crate::value::ValueTryIntoRef;
          Ok(
            $crate::functions::make_function_call!($function_name, Self::$f_name, arguments, $( $arg_type,)*)
            .map(|r| -> crate::value::Value { r.into() })?,
          )
        }
        else
        {
          Err(RunTimeError::InvalidNumberOfArguments { function_name: stringify!($function_name), got: arguments.len(), expected: ARG_COUNT }.into())
        }
      }
      fn validate_arguments(
        &self,
        args: Vec<$crate::compiler::expression_analyser::ExpressionType>,
      ) -> crate::Result<$crate::compiler::expression_analyser::ExpressionType>
      {
        let val_fn = $validator;
        val_fn(args)
      }
      fn is_deterministic(&self) -> bool
      {
        true
      }
    }
  };
}

macro_rules! declare_function {
  ($function_name: ident, $type_name: ty, $f_name: ident (  $( $arg_type: ty $(,)? )* ) -> $ret_type: ty ) => {
    $crate::functions::declare_function_!($function_name, $type_name,
      $f_name (  $( $arg_type, )* ) -> $ret_type, false,
      {$crate::functions::default_validate_!($function_name, $ret_type)} );
  };
  ($function_name: ident, $type_name: ty, $f_name: ident (  $( $arg_type: ty $(,)? )* ) -> $ret_type: ty, accept_null ) => {
    $crate::functions::declare_function_!($function_name, $type_name,
      $f_name (  $( $arg_type, )* ) -> $ret_type, true,
      {$crate::functions::default_validate_!($function_name, $ret_type)} );
  };
  ($function_name: ident, $type_name: ty, $f_name: ident (  $( $arg_type: ty $(,)? )* ) -> $ret_type: ty, validate_args( $( $expression_types: pat ),+ ) ) => {
    $crate::functions::declare_function_!($function_name, $type_name,
      $f_name (  $( $arg_type, )* ) -> $ret_type, false,
      {$crate::functions::validate_args_!($function_name, $ret_type, $( $expression_types ),+ )} );
  };
  ($function_name: ident, $type_name: ty, custom_trait ) => {
    impl $type_name
    {
      pub(super) fn create() -> (String, crate::functions::Function)
      {
        (
          stringify!($function_name).to_string(),
          std::sync::Arc::new(Box::new(Self {})),
        )
      }
    }
  };
}

pub(crate) use count_arguments;
pub(crate) use count_patterns;
pub(crate) use declare_function;
pub(crate) use declare_function_;
pub(crate) use default_validate_;
pub(crate) use make_function_argument;
pub(crate) use make_function_call;
pub(crate) use validate_args_;
