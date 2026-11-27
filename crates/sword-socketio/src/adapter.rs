use socketioxide::adapter::LocalAdapter;

pub type DefaultAdapter = LocalAdapter;

pub type DefaultAdapterState = ();

pub const ADAPTER_NAME: &str = "local";
