extern crate discord;
extern crate libc;

#[macro_use]
mod macros;
mod ffi;
mod types;
mod util;
mod connection;
mod message;
mod event_proc;

use ffi::*;
use connection::*;

pub use ffi::wdr_init;
pub use ffi::wdr_end;

mod weechat {
    pub const COMMAND: &'static str = "discord";
    pub const DESCRIPTION: &'static str = "\
Discord from the comfort of your favorite command-line IRC client!
This plugin is a work in progress and could use your help.
Check it out at https://github.com/khyperia/weechat-discord

Options used:

plugins.var.weecord.token = <discord_token>
plugins.var.weecord.on_delete.<server_id> = <channel_id>
plugins.var.weecord.rename.<id> = <string>
";
    pub const ARGS: &'static str = "\
                     connect
                     disconnect
                     token <token>";
    pub const ARGDESC: &'static str = "\
connect: sign in to discord and open chat buffers
disconnect: sign out of Discord and close chat buffers
token: set Discord login token

Example:
  /discord token 123456789ABCDEF
  /discord connect
";
    pub const COMPLETIONS: &'static str = "connect || disconnect || token || debug replace";
}

// *DO NOT* touch this outside of init/end
static mut MAIN_COMMAND_HOOK: *mut HookCommand = 0 as *mut _;

// Called when plugin is loaded in Weechat
pub fn init() -> Option<()> {
    let hook = tryopt!(ffi::hook_command(weechat::COMMAND,
                                         weechat::DESCRIPTION,
                                         weechat::ARGS,
                                         weechat::ARGDESC,
                                         weechat::COMPLETIONS,
                                         move |buffer, input| run_command(buffer, input)));
    unsafe {
        MAIN_COMMAND_HOOK = Box::into_raw(Box::new(hook));
    };
    Some(())
}

// Called when plugin is unloaded from Weechat
pub fn end() -> Option<()> {
    unsafe {
        let _ = Box::from_raw(MAIN_COMMAND_HOOK);
        MAIN_COMMAND_HOOK = ::std::ptr::null_mut();
    };
    Some(())
}

fn user_set_option(name: &str, value: &str) {
    command_print(&ffi::set_option(name, value));
}

fn command_print(message: &str) {
    MAIN_BUFFER.print(&format!("{}: {}", &weechat::COMMAND, message));
}

fn run_command(buffer: Buffer, command: &str) {
    let _ = buffer;
    if command == "" {
        command_print("see /help discord for more information")
    } else if command == "connect" {
        match ffi::get_option("token") {
            Some(t) => MyConnection::create(t),
            None => {
                command_print("Error: plugins.var.weecord.token unset. Run:");
                command_print("/discord token 123456789ABCDEF");
                return;
            }
        };
    } else if command == "disconnect" {
        MyConnection::drop();
        command_print("disconnected");
    } else if command.starts_with("token ") {
        user_set_option("token", &command["token ".len()..]);
    } else if command.starts_with("debug ") {
        debug_command(&command["debug ".len()..]);
    } else {
        command_print("unknown command");
    }
}
