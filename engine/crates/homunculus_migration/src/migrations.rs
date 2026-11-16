//! The startup migrations, one plugin per migration.

mod remove_app_exit_mod;

pub(crate) use remove_app_exit_mod::RemoveAppExitModPlugin;
