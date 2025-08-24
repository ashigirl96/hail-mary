## Document

Plugin API - Events
A plugin can subscribe to multiple Events. These events will be sent to the plugin through its update method.

For more detailed information, please see the zellij-tile API documentation.

ModeUpdate
Requires the ReadApplicationState permission
Provides information about the input modes of Zellij (eg. Normal, Locked, Pane, Tab, etc.). It also provides information about the bound keys, the style (the active theme colors) and the session name.

TabUpdate
Requires the ReadApplicationState permission
Provides information about the active tabs in Zellij, their position, name, whether they contain a pane in full screen, how many hidden panes they contain and information on the swap layouts.

PaneUpdate
Requires the ReadApplicationState permission
Provides information about the active panes in Zellij, their title, command and exit code (if available), etc.

SessionUpdate
Requires the ReadApplicationState permission
Provides information about the active sessions (of the current version) running on the machine.

Key
A user pressed a key when focused to this plugin, this event also provides the key pressed.

Mouse
A user issued a mouse action (click, scroll, etc.) while focused on the plugin, this event also provides the action in question.

Timer
This event is fired when a timer the plugin set is expired. This corresponds to the set_timeout plugin command;

CopyToClipboard
Requires the ReadApplicationState permission
This event is fired when the user copies a String to their clipboard

SystemClipboardFailure
Requires the ReadApplicationState permission
This event is fired when the user fails to copy a String to their clipboard

InputReceived
This event is fired whenever any input is received in Zellij, but does not specify which input

Visible
This event is fired when the current plugin becomes visible or invisible (eg. when switching a tab to and away from it).

CustomMessage
This event corresponds to the post_message_to and post_message_to_plugin plugin commands, used for a plugin and its workers to communicate. For more information, please see: Workers for Async Tasks.

FileSystemCreate, FileSystemRead, FileSystemUpdate, FileSystemDelete
These events are fired when the user creates a file, reads a file, updates a file or deletes a file in the folder in which Zellij was started. It includes a vector of the files in question.

RunCommandResult
Returned after the RunCommand plugin command. Containing the exit status, STDIN and STDOUT of the command as well as the context (an arbitrary string dictionary) provided when initiating the command.

WebRequestResult
Returned after the WebRequest plugin command. Containing the status code and body of the request as well as the context (an arbitrary string dictionary) provided when initiating the command.

CommandPaneOpened
Requires the ReadApplicationState permission
Returned after a pane opened with the OpenCommandPane plugin command is opened. Contains the terminal pane id of the pane, the context (an arbitrary string dictionary) provided when initiating the command.

CommandPaneExited
Requires the ReadApplicationState permission
Returned after a pane opened with the OpenCommandPane plugin command has exited. Note that this does not mean the pane is closed, it only means the command inside it has exited. This can happen multiple times if (for example) the user reruns the command by pressing Enter when focused on the command pane. Contains the terminal pane id of the pane, the command's numeric exit code (if there was one) as well as the context (an arbitrary string dictionary) provided when initiating the command.

PaneClosed
Requires the ReadApplicationState permission
A pane inside the current session was closed. Includes the pane's id.

EditPaneOpened
Requires the ReadApplicationState permission
Returned after a pane opened with the OpenFile plugin command is opened. Contains the terminal pane id of the editor pane, the context (an arbitrary string dictionary) provided when initiating the command.

EditPaneExited
Requires the ReadApplicationState permission
Returned after a pane opened with the OpenFile plugin command has exited. Contains the terminal pane id of the editor pane, the editor's numeric exit code (if there was one) as well as the context (an arbitrary string dictionary) provided when initiating the command.

CommandPaneReRun
Requires the ReadApplicationState permission
Returned after a pane opened with the OpenCommandPane plugin command has been re-run. This can happen multiple times and is often (but not necessarily) a result of the user pressing Enter when focused on the command pane. Contains the terminal pane id of the pane, the command's numeric exit code (if there was one) as well as the context (an arbitrary string dictionary) provided when initiating the command.

FailedToWriteConfigToDisk
Requires the ReadApplicationState permission
After the plugin attempted writing the configuration to disk (with the Reconfigure plugin command) and there was an error (eg. the file was read-only), this event is sent - optionally with the relevant error.

ListClients
The result of the ListClients plugin command. Contains information about all connected clients in the session, including their id, their focused pane id, the stringified representation of the running command or plugin inside their focused pane (if any), as well as an indication of whether they are the current client or not.

PastedText
The user just pasted the given text while focused on the plugin.

ConfigWasWrittenToDisk
A new configuration was successfully saved in the configuration file listened to by the current session.

WebServerStatus
This event is sent as a reply to the QueryWebServer command. It can be either online (and include the base url), offline or different_version (including the specified version).

FailedToStartWebServer
This event is sent as a reply to the StartWebServer command, when Zellij failed to start the web server. It includes a String representing the error.

BeforeClose
This event (if subscribed to) is called before a plugin is being unloaded, and is a chance for a plugin to do some cleanups.

InterceptedKeyPress
This event is similar to the Key Event, but represent a keypress that was intercepted after the InterceptKeyPresses plugin command was issued.

Plugin API - Commands
Zellij exports functions that allow plugins to control Zellij or change its behavior.

For more exact information, please see the zellij-tile API documentation.

subscribe
This method is given a list of events that the plugin is interested in. The plugin's update method will be called with the events and its payload, if any.

unsubscribe
Same as subscribe, only removes subscriptions to events.

request_permission
This command should be run in the load method of the plugin lifecycle, and contain one or more PermissionTypes. This will ask the user to provide the plugin said permissions.

set_selectable
Sets the plugin as selectable or unselectable to the user. Unselectable plugins might be desired when they do not accept user input.

get_plugin_ids
Returns the unique Zellij pane ID for the plugin as well as the Zellij process id.

get_zellij_version
Returns the version of the running Zellij instance - can be useful to check plugin compatibility

open_file
Requires the OpenFiles permission
Open a file in the user's default $EDITOR in a new pane

open_file_floating
Requires the OpenFiles permission
Open a file in the user's default $EDITOR in a new floating pane

open_file_in_place
Requires the OpenFiles permission
Open a file in the user's default $EDITOR, temporarily replacing the focused pane

open_file_with_line
Requires the OpenFiles permission
Open a file to a specific line in the user's default $EDITOR (if it supports it, most do) in a new pane

open_file_with_line_floating
Requires the OpenFiles permission
Open a file to a specific line in the user's default $EDITOR (if it supports it, most do) in a new floating pane

open_file_near_plugin
Requires the OpenFiles permission
Open a file in the user's default $EDITOR in the same tab as the plugin as a tiled pane, regardless of the user's focus

open_file_floating_near_plugin
Requires the OpenFiles permission
Open a file in the user's default $EDITOR in the same tab as the plugin as a floating pane, regardless of the user's focus

open_file_in_place_of_plugin
Requires the OpenFiles permission
Open a file in the user's default $EDITOR, temporarily replacing the plugin, regardless of the user's focus.

open_terminal
Requires the OpenTerminalsOrPlugins permission
Open a new terminal pane to the specified location on the host filesystem

open_terminal_floating
Requires the OpenTerminalsOrPlugins permission
Open a new floating terminal pane to the specified location on the host filesystem

open_terminal_in_place
Requires the OpenTerminalsOrPlugins permission
Open a new terminal pane to the specified location on the host filesystem, temporarily replacing the focused pane

open_terminal_near_plugin
Requires the OpenTerminalsOrPlugins permission
Open a new tiled terminal in the tab where the plugin resides, regardless of the user's focus.

open_terminal_floating_near_plugin
Requires the OpenTerminalsOrPlugins permission
Open a new floating terminal in the tab where the plugin resides, regardless of the user's focus.

open_terminal_in_place_of_plugin
Requires the OpenTerminalsOrPlugins permission
Open a new terminal on top of the plugin, temporarily replacing it. Regardless of the user's focus.

open_command_pane
Requires the RunCommands permission Open a new command pane with the specified command and args (this sort of pane allows the user to control the command, re-run it and see its exit status through the Zellij UI).
open_command_pane_floating
Requires the RunCommands permission
Open a new floating command pane with the specified command and args (this sort of pane allows the user to control the command, re-run it and see its exit status through the Zellij UI).

open_command_pane_in_place
Requires the RunCommands permission
Open a new command pane with the specified command and args (this sort of pane allows the user to control the command, re-run it and see its exit status through the Zellij UI), temporarily replacing the focused pane

open_command_pane_near_plugin
Requires the RunCommands permission
Open a new command pane with the specified command and args (this sort of pane allows the user to control the command, re-run it and see its exit status through the Zellij UI), as a tiled pane in the same tab as the plugin, regardless of the user's focus.

open_command_pane_floating_near_plugin
Requires the RunCommands permission
Open a new command pane with the specified command and args (this sort of pane allows the user to control the command, re-run it and see its exit status through the Zellij UI), as a floating pane in the same tab as the plugin, regardless of the user's focus.

open_command_pane_in_place_of_plugin
Requires the RunCommands permission
Open a new command pane with the specified command and args (this sort of pane allows the user to control the command, re-run it and see its exit status through the Zellij UI), on top of the plugin, temporarily replacing it, regardless of the user's focus.

run_command
Requires the RunCommands permission Run this host command in the background on the host machine, optionally being notified of its output if subscribed to the RunCommandResult Event. This API method includes a dictionary of arbitrary strings that will be returned verbatim with the RunCommandResult event. It can be used for things such as "request_id" to be able to identify the output of a command, or whatever else is needed.
web_request
Requires the WebAccess permission Make a web request, optionally being notified of its output if subscribed to the WebRequestResult Event. This API method includes a dictionary of arbitrary strings that will be returned verbatim with the WebRequestResult event. It can be used for things such as "request_id" to be able to identify the output of a command, or whatever else is needed.
switch_tab_to
Change the focused tab to the specified index (corresponding with the default tab names, to starting at 1, 0 will be considered as 1).

set_timeout
Set a timeout in seconds (or fractions thereof) after which the plugins update method will be called with the Timer event. Be sure to subscribe to it beforehand!

hide_self
Hide the plugin pane (suppress it) from the UI

show_self
Show the plugin pane (unsuppress it if it is suppressed), focus it and switch to its tab

switch_to_input_mode
Requires the ChangeApplicationState permission
Switch to the specified Input Mode (eg. Normal, Tab, Pane)

new_tabs_with_layout
Requires the ChangeApplicationState permission
Provide a stringified layout to be applied to the current session. If the layout has multiple tabs, they will all be opened.

new_tabs_with_layout_info
Requires the ChangeApplicationState permission
Provide a layout name or file path to be applied to the current session. If the layout has multiple tabs, they will all be opened.

new_tab
Requires the ChangeApplicationState permission
Open a new tab with the default layout

go_to_next_tab
Requires the ChangeApplicationState permission
Change focus to the next tab or loop back to the first

go_to_previous_tab
Requires the ChangeApplicationState permission
Change focus to the previous tab or loop back to the last

resize_focused_pane
Requires the ChangeApplicationState permission
Either Increase or Decrease the size of the focused pane

resize_focused_pane_with_direction
Requires the ChangeApplicationState permission
Either Increase or Decrease the size of the focused pane in a specified direction (eg. Left, Right, Up, Down).

focus_next_pane
Requires the ChangeApplicationState permission
Change focus tot he next pane in chronological order

focus_previous_pane
Requires the ChangeApplicationState permission
Change focus to the previous pane in chronological order

move_focus
Requires the ChangeApplicationState permission
Change the focused pane in the specified direction

move_focus_or_tab
Requires the ChangeApplicationState permission
Change the focused pane in the specified direction, if the pane is on the edge of the screen, the next tab is focused (next if right edge, previous if left edge).

detach
Requires the ChangeApplicationState permission
Detach the user from the active session

edit_scrollback
Requires the ChangeApplicationState permission
Edit the scrollback of the focused pane in the user's default $EDITOR

write
Requires the WriteToStdin permission
Write bytes to the STDIN of the focused pane

write_chars
Requires the WriteToStdin permission
Write characters to the STDIN of the focused pane

toggle_tab
Requires the ChangeApplicationState permission
Focused the previously focused tab (regardless of the tab position)

move_pane
Requires the ChangeApplicationState permission
Switch the position of the focused pane with a different pane

move_pane_with_direction
Requires the ChangeApplicationState permission
Switch the position of the focused pane with a different pane in the specified direction (eg. Down, Up, Left, Right).

clear_screen
Requires the ChangeApplicationState permission
Clear the scroll buffer of the focused pane

scroll_up
Requires the ChangeApplicationState permission
Scroll the focused pane up 1 line

scroll_down
Requires the ChangeApplicationState permission
Scroll the focused pane down 1 line

scroll_to_top
Requires the ChangeApplicationState permission
Scroll the focused pane all the way to the top of the scrollbuffer

scroll_to_bottom
Requires the ChangeApplicationState permission
Scroll the focused pane all the way to the bottom of the scrollbuffer

page_scroll_up
Requires the ChangeApplicationState permission
Scroll the focused pane up one page

page_scroll_down
Requires the ChangeApplicationState permission
Scroll the focused pane down one page

toggle_focus_fullscreen
Requires the ChangeApplicationState permission
Toggle the focused pane to be fullscreen or normal sized

toggle_pane_frames
Requires the ChangeApplicationState permission
Toggle the UI pane frames on or off

toggle_pane_embed_or_eject
Requires the ChangeApplicationState permission
Embed the currently focused pane (make it stop floating) or turn it to a float pane if it is not

close_focus
Requires the ChangeApplicationState permission
Close the focused pane

toggle_active_tab_sync
Requires the ChangeApplicationState permission
Turn the STDIN synchronization of the current tab on or off

close_focused_tab
Requires the ChangeApplicationState permission
Close the focused tab

quit_zellij
Requires the ChangeApplicationState permission
Compeltely quit Zellij for this and all other connected clients

previous_swap_layout
Requires the ChangeApplicationState permission
Change to the previous swap layout

next_swap_layout
Requires the ChangeApplicationState permission
Change to the next swap layout

go_to_tab_name
Requires the ChangeApplicationState permission
Change focus to the tab with the specified name

focus_or_create_tab
Requires the ChangeApplicationState permission
Change focus to the tab with the specified name or create it if it does not exist

post_message_to
Post a message to a worker of this plugin, for more information please see Plugin Workers

post_message_to_plugin
Post a message to this plugin (usually used to communicate with the worker), for more information, please see Plugin Workers

close_terminal_pane
Requires the ChangeApplicationState permission
Closes a terminal pane with the specified id

close_plugin_pane
Requires the ChangeApplicationState permission
Closes a plugin pane with the specified id

focus_terminal_pane
Requires the ChangeApplicationState permission
Changes the focus to the terminal pane with the specified id, unsuppressing it if it was suppressed and switching to its tab and layer (eg. floating/tiled).

focus_plugin_pane
Requires the ChangeApplicationState permission
Changes the focus to the plugin pane with the specified id, unsuppressing it if it was suppressed and switching to its tab and layer (eg. floating/tiled).

rename_terminal_pane
Requires the ChangeApplicationState permission
Changes the name (the title that appears in the UI) of the terminal pane with the specified id.

rename_plugin_pane
Requires the ChangeApplicationState permission
Changes the name (the title that appears in the UI) of the plugin pane with the specified id.

rename_tab
Requires the ChangeApplicationState permission
Changes the name (the title that appears in the UI) of the tab with the specified position.

switch_session
Requires the ChangeApplicationState permission
Change the session to the specified one, creating it if it does not exist

switch_session_with_focus
Requires the ChangeApplicationState permission
Change the session to the specified one (creating it if it does not exist), if it does exist - focusing on a tab or a pane inside that session

switch_session_with_layout
Requires the ChangeApplicationState permission
Change the session to the specified one, creating it if it does not exist, using a specified layout and optionally also a cwd (working directory).

block_cli_pipe_input
Requires the ReadCliPipes permission
Block the input side of a pipe, will only be released once this or another plugin unblocks it

(By default, pipes are unblocked after a plugin has handled a message unless this method is explicitly called).

unblock_cli_pipe_input
Requires the ReadCliPipes permission
Unblock the input side of a pipe, requesting the next message be sent if there is one

cli_pipe_output
Requires the ReadCliPipes permission
Send output to the output side of a pipe, ths does not affect the input side of same pipe

pipe_message_to_plugin
Requires the MessageAndLaunchOtherPlugins permission
Send a message to a plugin, it will be launched if it is not already running

delete_dead_session
Requires the ChangeApplicationState permission
Delete a dead session (one that is not running but can be resurrected) with a specific name

delete_all_dead_sessions
Requires the ChangeApplicationState permission
Delete all dead sessions (sessions that are not running but can be resurrected)

rename_session
Requires the ChangeApplicationState permission
Rename the current session to a specific name

disconnect_other_clients
Requires the ChangeApplicationState permission
Disconnect all other clients attached to this session except the one making this call

kill_sessions
Requires the ChangeApplicationState permission
Kill all Zellij sessions in the list (can contain one or more session names)

scan_host_folder
This is a stop-gap method that allows plugins to scan a folder on the /host filesystem and get back a list of files. The reason this is done through the API is that at the time of development, doing this through our WASI runtime is extremely slow. We hope this method will not be needed in the future.

dump_session_layout
Requires the ReadApplicationState permission
Request Zellij send back the serialized layout (in KDL format) of the current session. The layout will be sent back as a CustomMessage with the session_layout name and the stringified layout as the message payload.

close_self
Will close the plugin and its pane. If the plugin is the only selectable pane in the session, the session will also exit.

reconfigure
Requires the Reconfigure permission
Provide a stringified configuration to be "merged" with the configuration of the current session. Optionally also written to disk and so applied to all other sessions listening to the same configuration file.

Use this command to bind global keys to the plugin
It's possible to use the reconfigure command to bind the special MessagePluginId temporary keybinding (which will not be saved to disk). This keybind, along with the plugin's id obtained from get_plugin_ids can be used to bind a user key to trigger this plugin with a pipe.

Example:

let config = format!(r#"
keybinds {{
    shared {{
        bind "Ctrl Shift r" {{
            MessagePluginId {} {{
                name "my_message_name"
            }}
        }}
    }}
}}"#);
reconfigure(config, false)
// now, whenever a user pressed `Ctrt Shift r` anywhere in the app, the plugin's pipe method will trigger with the "my_message_name" message
hide_pane_with_id
Requires the ChangeApplicationState permission
Hide a pane (suppress id) with the specified id.

show_pane_with_id
Requires the ChangeApplicationState permission
Show a pane with the specified id, unsuppress it if it is suppressed, focus it and switch to its tab.

open_command_pane_background
Requires the RunCommands permission
Open a new hidden (background) command pane with the specified command and args (this sort of pane allows the user to control the command, re-run it and see its exit status through the Zellij UI).

rerun_command_pane
Requires the RunCommands permission
Re-run command in a command pane (similar to a user focusing on it and pressing <ENTER>).

resize_pane_with_id
Requires the ChangeApplicationState permission
Change the size of the specified pane (optionally in a specific direction).

edit_scrollback_for_pane_with_id
Requires the ChangeApplicationState permission
Edit the scrollback of the specified pane in the user's default $EDITOR

write_to_pane_id
Requires the WriteToStdin permission
Write bytes to the STDIN of the specified pane

write_chars_to_pane_id
Requires the WriteToStdin permission
Write characters to the STDIN of the specified pane

move_pane_with_pane_id
Requires the ChangeApplicationState permission
Switch the position of the specified pane with a different pane

move_pane_with_pane_id_in_direction
Requires the ChangeApplicationState permission
Switch the position of the specified pane with a different pane in the specified direction (eg. Down, Up, Left, Right).

clear_screen_for_pane_id
Requires the ChangeApplicationState permission
Clear the scroll buffer of the specified pane

scroll_up_in_pane_id
Requires the ChangeApplicationState permission
Scroll the specified pane up 1 line

scroll_down_in_pane_id
Requires the ChangeApplicationState permission
Scroll the specified pane down 1 line

scroll_to_top_in_pane_id
Requires the ChangeApplicationState permission
Scroll the specified pane all the way to the top of the scrollbuffer

scroll_to_bottom
Requires the ChangeApplicationState permission
Scroll the specified pane all the way to the bottom of the scrollbuffer

page_scroll_up_in_pane_id
Requires the ChangeApplicationState permission
Scroll the specified pane up one page

page_scroll_down_in_pane_id
Requires the ChangeApplicationState permission
Scroll the specified pane down one page

toggle_pane_id_fullscreen
Requires the ChangeApplicationState permission
Toggle the specified pane to be fullscreen or normal sized

toggle_pane_embed_or_eject_for_pane_id
Requires the ChangeApplicationState permission
Embed the specified pane (make it stop floating) or turn it to a float pane if it is not

close_tab_with_index
Requires the ChangeApplicationState permission
Close the focused tab

break_panes_to_new_tab
Requires the ChangeApplicationState permission
Create a new tab that includes the specified pane ids

break_panes_to_tab_with_index
Requires the ChangeApplicationState permission
Move the specified pane ids to the tab with the specified index

reload_plugin
Requires the ChangeApplicationState permission
Reload the plugin with the specified id

load_new_plugin
Requires the ChangeApplicationState permission
Load a new plugin

rebind_keys
Requires the Reconfigure permission
Given a set of keys to unbind and a set of keys to bind (in that order), this will apply them to the current session - or optionally also save them to the configuration file.

list_clients
Requires the ReadApplicationState permission
List information about clients connected to this session. Including their ID, their focused pane id, the command or plugin running in that pane id (if any) and whether they are the current plugin. This will be returned as the ListClients Event that must be subscribed to beforehand.

change_host_folder
Requires the FullHdAccess permission
Change the location of the /host folder from the perspective of the plugin to somewhere else on the filesystem.

set_floating_pane_pinned
Requires the ChangeApplicationState permission
Make a floating pane pinned or unpinned (always on top).

stack_panes
Requires the ChangeApplicationState permission
Given a list of pane ids, turns them into a stack.

change_floating_panes_coordinates
Requires the ChangeApplicationState permission
Given a list of pane ids and corresponding coordinates (x, y, width, height) will change the location of all of these IDs to the desired coordinates.

group_and_ungroup_panes
Requires the ChangeApplicationState permission
Accepts two lists of panes, first one to group, second one to ungroup (in this logical order). Grouping is performed for the benefit of the "multiple-select" functionality.

highlight_and_unhighlight_panes
Requires the ChangeApplicationState permission
Accepts two lists of panes, first one to group, second one to ungroup (in this logical order). The highlight is cosmetic and is meant to help mark panes.

close_multiple_panes
Requires the ChangeApplicationState permission
Accepts a list of pane ids to close.

float_multiple_panes
Requires the ChangeApplicationState permission
Accepts a list of pane ids to make floating (ignores panes that are already floating).

embed_multiple_panes
Requires the ChangeApplicationState permission
Accepts a list of pane ids to embed (not floating). Ignores panes that are already floating.

start_web_server
Requires the StartWebServer permission
Start the Zellij web-server.

stop_web_server
Requires the StartWebServer permission
Stop the Zellij web-server.

share_current_session
Requires the StartWebServer permission
Allows the current session to be shared (attached to) on the Zellij web-server.

stop_sharing_current_session
Requires the StartWebServer permission
Removes permission for the current session to be shared (attached to) on the Zellij web-server, also disconnects current web clients.

query_web_server_status
Requires the StartWebServer permission
Queries the status of the Zellij web-server, response will be returned as the WebServerStatus event (which must also be subscribed to).

generate_web_login_token
Requires the StartWebServer permission
Generates (and returns) a new web login token, optionally with a provided name as a String. (This token is hashed in a local DB, so can never be displayed again).

revoke_web_login_token
Requires the StartWebServer permission
Revoked an existing web login token by its name.

revoke_all_web_login_tokens
Requires the StartWebServer permission
Revokes all web login tokens.

list_web_login_tokens
Requires the StartWebServer permission
Returns a list of existing web login tokens (their names, the tokens themselves cannot be displayed) and their creation times.

rename_web_login_token
Requires the StartWebServer permission
Rename a web login token by providing its existing name.

intercept_key_presses
Requires the InterceptInput permission
Intercept all user input, having it sent to the plugin as an InterceptedKeyPress event

clear_key_presses_intercepts
Requires the InterceptInput permission
Clear the interception of key presses, having them return to being sent to the application itself. This happens automatically when the plugin is unloaded.

replace_pane_with_existing_pane
Requires the ChangeApplicationState permission
Replaces a specific pane (denoted by is PaneId) with another existing pane (also denoted by its PaneId)


Permissions
The plugin system provides a permission system to provide extra security and protection to the user.

The system places certain Events and Commands behind certain permissions. Plugins who want to listen to these events or use these commands should prompt the user to grant them these permissions with the request_permission command.

Permissions
ReadApplicationState
Access Zellij state (Panes, Tabs and UI)

ChangeApplicationState
Change Zellij state (Panes, Tabs and UI)

OpenFiles
Open files (eg. for editing)

RunCommand
Run commands in panes or silently

OpenTerminalsOrPlugins
Start new terminals and plugins

WriteToStdin
Write to STDIN as if it were the user

Reconfigure
Change the configuration (running and also saved in the configuration file) of Zellij.

FullHdAccess
Access the full HD on the machine rather than just the folder in which Zellij was started.

StartWebServer
Control (start, stop, get status, manage login tokens) the Zellij web-server

InterceptInput
Intercept user input (eg. keypresses), having all of this input sent to the plugin instead.


Plugin API - Configuration
Plugins can be configured (have their behavior changed when instantiated) with an arbitrary key/value list. This configuration is available to plugins in their load method. It can be provided through layouts:

    pane {
        plugin location="file:/path/to/my/plugin.wasm" {
            some_key "some_value"
            another_key 1
        }
    }
Or through the command line:

zellij action launch-or-focus-plugin --configuration "some_key=some_value,another_key=1"

Plugin API - Reading from the Filesystem
Plugins can use their own native standard library to read from the filesystem.

eg.

std::fs::write("/host/my_file.txt", "hi from a plugin!").unwrap()
Zellij maps three paths for each plugin:

/host - the cwd of the last focused terminal, or the folder where Zellij was started if that's not available
/data - its own folder, shared with all loaded instances of the plugin - created on plugin load and deleted on plugin unload.
/tmp - a temporary folder located in an arbitrary position in the system's temporary filesystem.

Plugin API - Logging
Whatever plugins print to their STDERR will be logged in the zellij log.

The Zellij log is located at: /$temp_dir/zellij-<UID>/zellij-log/zellij.log. $temp_dir, in most systems will be /tmp, but there can be exceptions, such as /var/folders/dr/xxxxxxxxxxxxxx/T/ for Mac.


Plugin Workers
Plugin workers are a way to get around the fact that wasm/wasi threads are not stable yet. If a plugin has a potentially long operation to perform, it can declare a worker on startup and send and receive messages from it.

The ZellijWorker trait
zellij-tile provides the following interface for workers:

pub trait ZellijWorker<'de>: Default + Serialize + Deserialize<'de> {
    fn on_message(&mut self, message: String, payload: String) {}
}
The on_message method will be called when the plugin uses the post_message_to plugin command with an arbitrary message and payload. These are specified as Strings so that plugins can decide on their own method of serialization.

Registering Workers
To register workers on startup, plugins can use the register_worker macro like so:


pub struct TestWorker {
    // ...
}
impl ZellijWorker for TestWorker {
    // ...
}
register_worker!(
    TestWorker,
    test_worker, // the namespace of the worker, anything before the final _worker will be the worker namespace
    TEST_WORKER // a name for static variable used to store the worker state between invocations
);
For more information, please see the zellij-tile API documentation.

Sending messages to workers
When a plugin (or another worker) wishes to send messages to a worker, they use the post_message_to plugin command. They should use the worker namespace used when registering the worker, eg. post_message_to("test", ...) for the test_worker example above.

Sending messages from workers to plugins
When a worker wishes to send a message to a plugin, they use the post_message_to_plugin command. This message will trigger the plugin's update method with a CustomMessage event. Be sure to subscribe to it beforehand!

Pipes for Communicating with and Between plugins
What are pipes?
A Zellij pipe is a unidirectional communication channel to and/or from a plugin. This communication channel is used to send one or more messages containing arbitrary serializable text, similar to how pipes work in most shells.

Pipes can have a name (arbitrary string), a payload (arbitrary stringifiable content) and arguments (a dictionary of arbitrary string to arbitrary string). All of these are optional.

Pipes that do not have a specific destination are broadcast to all plugins. The reason for this is in order to facilitate the creation of conventions such as the "notification" pipe that can be handled by multiple different plugins in potentially different ways.

Pipes that do not have a name will be assigned a random UUID as their name.

Pipe destinations
A pipe destination can be any plugin url (eg. https://example.com/my-plugin.wasm, file:/path/to/plugin.wasm, etc.) coupled with a plugin configuration. Two plugins with the same URL and different configurations will each be considered a unique plugin destination.

If a plugin has multiple instances (such as is the case when multiple users are attached to the same session), each instance will receive messages from a pipe directed at this plugin.

If a destination is specified for a pipe and no such plugin is running, this plugin will be loaded on first message (the pipe will wait until it is loaded and then send it the first message - see backpressure below).

When started from a plugin, a pipe destination can also be the internal unique Zellij id of a specific plugin. This is to facilitate two-way communication between two plugins - see Pipe sources below.

Pipe sources
Pipes can be started either from the CLI, from a keybinding or from another plugin. The source of the pipe will be specified to the plugin (see below). If the source is another plugin, the internal Zellij id of the source plugin will be provided (to allow the plugin to respond in a new pipe if needed).

If the source is the CLI, the internal pipe-id (a UUID) will be provided to allow plugins to apply backpressure to the CLI pipe as needed (for example, pausing a CLI pipeline until the user presses a specific key).

CLI pipes and backpressure
Pipes can be started from the CLI, in which case they can potentially listen to STDIN and send multiple messages down the same pipe. It's important to stress that this is usually slower than piping data to other programs, namely because Zellij plugins often render themselves on each pipe message. The STDIN buffer is only released after the plugin has been rendered (or has elected not to render itself) in order to apply backpressure.

Zellij plugins can also elect to entirely block the CLI pipe, releasing it later based on (for example) user input. The same pipe can be blocked/released from any plugin, so long as it knows the CLI pipe ID provided as the pipe source.

A plugin can also print to the CLI pipe's STDOUT (unrelated to the data it gets on STDIN) assuming it knows its ID. In fact, multiple plugins (or plugin instances) can print to the STDOUT of the same pipe if so desired.

For more on this, see block_cli_pipe_input, unblock_cli_pipe_input and cli_pipe_output.

The pipe lifecycle method
Plugins may listen to pipes by implementing the pipe lifecycle method. This method is called every time a message is sent over a pipe to this plugin (whether it's broadcast to all plugins or specifically directed at this one). It receives a PipeMessage containing the source of the pipe (CLI, another plugin or a keybinding), as well as information about said source (the plugin id or the CLI pipe id). The PipeMessage also contains the name of the pipe (explicitly provided by the user or a random UUID assigned by Zellij), its payload if it has one, its arguments and whether it is private or not (a private message is one directed specifically at this plugin rather than broadcast to all plugins).

Similar to the update method, the pipe lifecycle method returns a bool, true if it would like to render itself, in which case the render function will be called as normal.

Here's a small Rust example:

fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
    let mut should_render = false;
    match pipe_message.source {
        PipeSource::Cli(input_pipe_id) => {
            if let Some(payload) = pipe_message.payload {
                self.messages_from_cli.push(payload);
                should_render = true;
            }
            if self.paused {
                // backpressure, this will pause data from the CLI pipeline until the unblock_cli_pipe_input method will be called for this id
                // from this or another plugin
                block_cli_pipe_input(&input_pipe_id);
            }
            if self.should_print_to_cli_stdout {
                // this can happen anywhere, anytime, from multiple plugins and is not tied to data from STDIN
                // as long as the pipe is open, plugins with its ID can print arbitrary data to its STDOUT side, even if the input side is blocked
                cli_pipe_output(input_pipe_id, &payload);
            }
        }
        PipeSource::Plugin(source_plugin_id) => {
            // pipes can also arrive from other plugins
        }
    }
    should_render
}
The pipe_message_to_plugin plugin API command
This pipe_message_to_plugin API command allows plugins to start a new pipe to another plugin. It allows spcifying a pipe destination, name, payload, args and also some information to be used in case this message will end up launching a new plugin (for example, the pane title of the new plugin).

Here's a short Rust example:

pipe_message_to_plugin(
    MessageToPlugin::new("message_name")
        .with_plugin_url("https://example.com/my-plugin.wasm")
        .new_plugin_instance_should_have_pane_title("new_plugin_pane_title")
);
The special zellij:OWN_URL pipe destination
When plugins open pipes, they can use the special zellij:OWN_URL destination url. Zellij will replace this URL with the plugin's own URL. This is useful when plugins want to launch new instances of themselves and communicate with them (for example, in order for the plugin to play different roles or to create a visual pipeline with multiple panes on the user's screen).

It's important to remember though that if this is used, care needs to be taken to make sure the new plugin's configuration is different from the currently running one - otherwise Zellij will send this message back to the plugin (see plugin uniqueness above).


Plugin Development Environment
For Rust plugins, Zellij provides an example plugin that also includes a development environment for plugin developers.

This development environment is created by the following Zellij layout (truncated here for clarity)

// plugin-development-workspace.kdl
layout {
    // ...
    pane edit="src/main.rs"
    pane edit="Cargo.toml"
    pane command="bash" { // could also be done with watchexec or something similar
        args "-c" "cargo build && zellij action start-or-reload-plugin file:target/wasm32-wasi/debug/rust-plugin-example.wasm"
    }
    pane {
        plugin location="file:target/wasm32-wasi/debug/rust-plugin-example.wasm"
    }
    // ...
}
Please check the example repository for the full version

This layout is intended to be loaded into Zellij (either in a running session or in a new session), to load the user's default $EDITOR to the main.rs and Cargo.toml files, show the rendered plugin in a separate pane as well as the compilation and plugin hot-reload logs.

Zellij plugins can of course be developed out of the terminal as well.

Plugin Lifecycle
Zellij provides the zellij-tile crate to plugins to facilitate development.

The zellij-tile crate provides the ZellijPlugin trait:

pub trait ZellijPlugin {
    fn load(&mut self) {}
    fn update(&mut self, event: Event) -> bool {
        false
    } // return true if it should render
    fn render(&mut self, rows: usize, cols: usize) {}
}
Lifecycle Methods
load
Will be called when the plugin is loaded, this is a good place to subscribe to events that are interesting for this plugin.

update
Will be called with an Event if the plugin is subscribed to said event. If the plugin returns true from this function, Zellij will know it should be rendered and call its render function.

Since events are used for asynchronous communication between Zellij and the plugin, they do not follow a specific order. This means, that a plugin could receive certain events (like ModeUpdate) before the PermissionRequestResult event is received. Therefore the plugin should ensure, that dependencies within the plugin logic between certain events are handled correctly. An example for waiting for the PermissionRequestResult can be found in this great plug post

render
Will be called either after an update that requested it, or when the plugin otherwise needs to be re-rendered (eg. on startup, or when the plugin is resized). The rows and cols values represent the "content size" of the plugin (this will not include its surrounding frame if the user has pane frames enabled).

This function is expeted to print to STDOUT whatever the plugin would like to render inside its pane. For more information, see plugin ui rendering.

Registering a plugin
After implementing the trait on a struct, we'll need to use the register_plugin macro on it:

struct MyPlugin {
    // ...
}

impl ZellijPlugin for MyPlugin {
    // ...
}

register_plugin!(MyPlugin);
Zellij will then instantiate the plugin (using the Default implementation) and call it as needed.



Rendering a UI
Rendering ANSI through STDOUT
When a plugin's render function prints to STDOUT, Zellij treats the printed bytes as utf-8 ANSI. One can print to a Zellij plugin just like one could print to any terminal and have it rendered, with the following exception:

Every time the render function is called, the previous state of the terminal is cleared. This is in order to facilitate UI development without having to keep track of the previous state on screen. This behavior might be toggleable in the future.

Plugin developers are free to use whichever terminal UI libraries they wish in order to render a Zellij plugin. In the future Zellij might offer a UI library of its own as well as an integration with a few popular ones.

Using the Built-in UI Components
Zellij provides plugins with some built-in UI components that will fit the user's theme and preferences. These are cross-language components, interpreted through serialized STDOUT in the render function as a private terminal DCS extension. The various plugin SDKs provide wrappers to facilitate serialization. All of these wrappers should be used inside the render function

The Components
Table
table

Consists of a title line with an emphasis style and a grid of width-justified cells. Each cell can be styled individually (see Text below) and also marked as "selected". Marking adjacent cells as selected can create a "selected row" effect.

Example from the Rust SDK (renders the screeshot above):

let table = Table::new()
    .add_row(vec!["title1", "title2", "title3"])
    .add_styled_row(vec![Text::new("content 1").color_range(0, 1..5), Text::new("content 2").color_range(2, ..), Text::new("content 3")])
    .add_styled_row(vec![Text::new("content 11").selected(), Text::new("content 22").selected(), Text::new("content 33").selected()])
    .add_styled_row(vec![Text::new("content 111"), Text::new("content 222").selected(), Text::new("content 33")])
    .add_styled_row(vec![Text::new("content 11"), Text::new("content 22").selected(), Text::new("content 33")]);
print_table(table); // will print this table wherever the cursor may be at the moment
print_table_with_coordinates(table, 4, 5, None, None); // will print this table at x: 4, y: 5, the last two `Option`s are width/height
Ribbon
ribbon

Ribbons are the UI elements used for tabs in the Zellij tab bar and for modes in the Zellij status-bar. They can be selected, which would change their background color, and can contain styled text themselves (see Text below).

Example from the Rust SDK (renders the screenshot above):

print_ribbon_with_coordinates(Text::new("ribbon 1").color_range(0, 1..5), 0, 0, Some(12), None);
print_ribbon_with_coordinates(Text::new("ribbon 2").color_range(1, 1..5).selected(), 12, 0,  Some(12), None);
print_ribbon_with_coordinates(Text::new("ribbon 3").color_range(2, 1..5), 24, 0, Some(12), None);
print_ribbon_with_coordinates(Text::new("ribbon 4").color_range(3, 1..5), 36, 0,  Some(12), None);
Nested List
nested-list

A nested list is the UI element used in the Zellij session-manager. It is a list with possibility indented lines to an arbitrary level. Each line can be selected (multiple lines can be selected as well), and each line can be styled individually (see Text below).

Example from the Rust SDK (renders the screenshot above):

print_nested_list_with_coordinates(vec![
    NestedListItem::new("item 1 with some nice text...").color_range(1, ..).color_range(3, 10..25).color_indices(1, vec![8]),
    NestedListItem::new("item 2 with some more text").indent(1).color_range(0, 1..15).color_indices(1, vec![8]),
    NestedListItem::new("item 3 is a real eye opener").color_range(2, ..).color_range(3, 5..20).color_indices(1, vec![8]).selected(),
    NestedListItem::new("item 4 is just another item, really").indent(1).color_range(0, ..).color_range(1, 1..15).color_indices(1, vec![8]),
], 1, 1, None, None);
Text
text

While this element can be rendered on its own, it's mainly used inside other elements for styling.

A Text element can be selected - which will be interpreted in the context of the element it resides in, generally changing its background in one way or another. A Text element can also have indices. These indices can be one of 4 colors (preset depending on the user's theme) assigned to characters or ranges inside the element. This can be especially useful when incorporated with fuzzy finding.

Example from the Rust SDK (renders the screenshot above):

let text = Text::new("foo bar baz").selected().color_range(0, 0..=2).color_range(1, 3..=5).color_range(2, 7..=9);
print_text_with_coordinates(text, 0, 0, None, None);
The Protocol
Note: This section discusses the private DCS ANSI serialization protocol used to represent the above components. It could be of interest to SDK authors, but plugin developers are encouraged to use the SDK abstractions instead.

An example component can look like this: (<ESC>, represents the escape character)

<ESC>Pzribbon;27,91,49,109,60,27,91,51,56,59,53,59,57,109,110,27,91,51,57,59,51,56,59,53,59,48,109,62,32,82,69,83,73,90,69<ESC>\
The first part of the sequence, <ESC>Pz is the DCS representing the beginning of a Zellij UI element, followed by the clear-text element name. Following is a semi-colon (;) separated list of items to be interpreted according to context. In the above case there's only one item representing a utf-8 encoded byte-string which is the ribbon's contents (the bytes separated by commas). Finally, the string terminator <ESC>\ representing the end of the UI element.

Coordinates
Each component can have an optional coordinates string, placed as the first element in the semi-colon separated list directly after the component name. Example:

<ESC>Pzribbon;2/2/10/5;114,105,98,98,111,110,32,49<ESC>\
Here, the coordinate string 2/3/10/5 instructs us to render the ribbon at x: 2, y: 3, width: 10, height: 5. The width and height are optional, so may be empty (eg. 2/3//).

Selected
If a utf-8 separated byte list begins with a clear-text x, it will be considered "selected". Eg.

<ESC>Pzribbon;x114,105,98,98,111,110,32,49<ESC>\
Opaque
If a utf-8 separated byte list begins with a clear-text z (note: must follow Selected is both are present), it will be considered "opaque". Eg.

<ESC>Ptext;z114,105,98,98,111,110,32,49<ESC>\
This indicates that the UI component should use an opaque background, defaulting to the user's black theme color. Otherwise it will be considered transparent and use no background (when possible). Opaque components are best used as part of status bars, transparent components when one wishes to represent bare text (for example, in help text).

Indices
A utf-8 separated byte list can be preceded by a dollar ($) separated index list representing colored indices. Each element within the dollar separated list can contain zero or more indexes (separated by commas) which will be colored in the desired index color (the colors themselves being determined by the user's theme). Example:

<ESC>Pzribbon;2/2/10/;1,2,3,4$5,6$$7$114,105,98,98,111,110,32,49<ESC>\
Here, indices 1, 2, 3 and 4 will be colored in index color 0 while 5 and 6 will be colored in index color 1. Index color 2 is empty, so no elements will be colored using it, and element number 7 will be colored in index color 3.

Indentation
In the context of a Nested List, elements can be arbitrarily indented. This is done one or more pipe (|) characters preceding the utf-8 byte list. Example:

<ESC>Pznested_list;105,116,101,109,32,51;|105,116,101,109,32,52;||105,116,101,109,32,53,32,108,115<ESC>\
Each item in a Nested List is represented as a utf-8 byte array separated by semicolons. Here, the first item will not be indented, the second item will be indented once, and the third item will be indented twice.



Plugin Aliases
Plugin aliases are a dictionary between an arbitrary string (eg. filepicker) and a non-alias plugin url, with optional plugin configuration. They can be configured in the Zellij configuration file under the plugins block.

Here's the default aliases:

plugins {
    tab-bar location="zellij:tab-bar"
    status-bar location="zellij:status-bar"
    strider location="zellij:strider"
    compact-bar location="zellij:compact-bar"
    session-manager location="zellij:session-manager"
    welcome-screen location="zellij:session-manager" {
        welcome_screen true
    }
    filepicker location="zellij:strider" {
        cwd "/"
    }
}
With this plugins block, whenever the bare tab-bar is used to refer to a plugin (be it in a layout, from the command line, from a keybinding or from another plugin), Zellij will translate it to the internal zellij:tab-bar url. Whenever the bare filepicker url is used to refer to a plugin, Zellij will translate it to the built-in zellij:strider url will be used with the cwd "/" configuration.

Aliases can be added to this block or changed to swap the default built-in plugins to other implementations. Removing the default aliases entirely might cause Zellij not to function as expected.

When swapping the default aliases for custom plugins, it's important that these plugins implement the basic contract Zellij (and indeed, other plugins) expect of them. The following sections describe the contract for each default alias.

Here's an example on how to use the plugin alias in a layout:

layout {
  default_tab_template {
    children
    pane size=1 borderless=true {
      plugin location="compact-bar"
    }
  }
}
A note about cwd
When an alias defined a cwd for its plugin (such as the filepicker example above), Zellij will add the caller_cwd configuration parameter with the cwd of the focused pane in addition to the configured cwd above, instead of overriding the configured cwd of the plugin. This is so that plugins may provide a nicer user experience to their users and still have the desired cwd configuration of the alias.


## Example


use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use owo_colors::OwoColorize;
use zellij_tile::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Pane {
    pub pane_info: PaneInfo,
    pub tab_info: TabInfo,
}

impl fmt::Display for Pane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} | {}", self.tab_info.name, self.pane_info.title)
    }
}

//<--------- TODO: Replace with official functions once available

fn get_focused_tab(tab_infos: &Vec<TabInfo>) -> Option<TabInfo> {
    for tab in tab_infos {
        if tab.active {
            return Some(tab.clone());
        }
    }
    return None;
}

fn get_focused_pane(tab_position: usize, pane_manifest: &PaneManifest) -> Option<PaneInfo> {
    let panes = pane_manifest.panes.get(&tab_position);
    if let Some(panes) = panes {
        for pane in panes {
            if pane.is_focused & !pane.is_plugin {
                return Some(pane.clone());
            }
        }
    }
    None
}

//--------->

// ----------------------------------- Update ------------------------------------------------

fn get_valid_panes(
    panes: &Vec<Pane>,
    pane_manifest: &PaneManifest,
    tab_infos: &Vec<TabInfo>,
) -> Vec<Pane> {
    let mut new_panes: Vec<Pane> = Vec::default();
    for pane in panes.clone() {
        // Iterate over all panes, and find corresponding tab and pane based on id
        // update it in case the info has changed, and if they are not there do not add them.
        if let Some(tab_info) = tab_infos.get(pane.tab_info.position) {
            if let Some(other_panes) = pane_manifest.panes.get(&pane.tab_info.position) {
                if let Some(pane_info) = other_panes
                    .iter()
                    .find(|p| !p.is_plugin & (p.id == pane.pane_info.id))
                {
                    let pane_info = pane_info.clone();
                    let tab_info = tab_info.clone();
                    let new_pane = Pane {
                        pane_info,
                        tab_info,
                    };
                    new_panes.push(new_pane);
                }
            }
        }
    }
    new_panes
}

#[derive(Default)]
struct State {
    selected: usize,
    panes: Vec<Pane>,
    focused_pane: Option<Pane>,
    tab_info: Option<Vec<TabInfo>>,
    pane_manifest: Option<PaneManifest>,
}

impl State {
    fn select_down(&mut self) {
        self.selected = (self.selected + 1) % self.panes.len();
    }

    fn select_up(&mut self) {
        if self.selected == 0 {
            self.selected = self.panes.len() - 1;
            return;
        }
        self.selected = self.selected - 1;
    }

    fn sort_panes(&mut self) {
        self.panes.sort_by(|x, y| {
            (x.tab_info.position)
                .partial_cmp(&y.tab_info.position)
                .unwrap()
        });
    }

    /// Update panes updates the pane states based on the latest pane_manifest and tab_info
    fn update_panes(&mut self) -> Option<()> {
        // Update panes to filter our invalid panes (e.g. tab/pane was closed).
        let pane_manifest = self.pane_manifest.clone()?;
        let tab_info = self.tab_info.clone()?;
        let panes = get_valid_panes(&self.panes.clone(), &pane_manifest, &tab_info);
        self.panes = panes;

        // Update currently focused pane
        let tab_info = get_focused_tab(&tab_info)?;
        let pane_info = get_focused_pane(tab_info.position, &pane_manifest)?;
        self.focused_pane = Some(Pane {
            pane_info,
            tab_info,
        });

        // Set default location of selected idx to currently focused pane
        if let Some(focused_pane) = &self.focused_pane {
            for (idx,pane) in self.panes.iter().enumerate() {
                if pane.pane_info.id == focused_pane.pane_info.id {
                    self.selected = idx;
                }
            }
        }else{
            self.selected = 0;
        }

        Some(())
    }
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::RunCommands,
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[EventType::Key, EventType::TabUpdate, EventType::PaneUpdate]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::TabUpdate(tab_info) => {
                self.tab_info = Some(tab_info);
                self.update_panes();
                should_render = true;
            }
            Event::PaneUpdate(pane_manifest) => {
                self.pane_manifest = Some(pane_manifest);
                self.update_panes();
                should_render = true;
            }
            Event::Key(key) => match key.bare_key {
                BareKey::Char('A') => {
                    let current_pane_ids: Vec<u32> =
                        self.panes.iter().map(|p| p.pane_info.id).collect();
                    if let Some(pane_manifest) = &self.pane_manifest {
                        if let Some(tab_info) = &self.tab_info {
                            for (tab_position, panes) in &pane_manifest.panes {
                                if let Some(tab) =
                                    tab_info.iter().find(|t| t.position == *tab_position)
                                {
                                    for pane in panes {
                                        if !pane.is_plugin && !current_pane_ids.contains(&pane.id) {
                                            self.panes.push(Pane {
                                                pane_info: pane.clone(),
                                                tab_info: tab.clone(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    self.sort_panes();
                    should_render = true;
                    hide_self();
                }
                BareKey::Char('a') => {
                    let panes_ids: Vec<u32> = self.panes.iter().map(|p| p.pane_info.id).collect();
                    if let Some(pane) = &self.focused_pane {
                        if !panes_ids.contains(&pane.pane_info.id) {
                            self.panes.push(pane.clone());
                            self.sort_panes();
                        }
                    }
                    should_render = true;
                    hide_self();
                }
                BareKey::Char('d') => {
                    if self.selected < self.panes.len() {
                        self.panes.remove(self.selected);
                    }
                    if self.panes.len() > 0 {
                        self.select_up();
                    }
                    should_render = true;
                }
                BareKey::Char('c') | BareKey::Esc => {
                    hide_self();
                }
                BareKey::Down | BareKey::Char('j') => {
                    if self.panes.len() > 0 {
                        self.select_down();
                        should_render = true;
                    }
                }
                BareKey::Up | BareKey::Char('k') => {
                    if self.panes.len() > 0 {
                        self.select_up();
                        should_render = true;
                    }
                }
                BareKey::Enter | BareKey::Char('l') => {
                    let pane = self.panes.get(self.selected);

                    if let Some(pane) = pane {
                        hide_self();
                        // TODO: This has a bug on macOS with hidden panes
                        focus_terminal_pane(pane.pane_info.id, true);
                    }
                }
                _ => (),
            },
            _ => (),
        };

        should_render
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        println!(
            "{}",
            self.panes
                .iter()
                .enumerate()
                .map(|(idx, pane)| {
                    if idx == self.selected {
                        pane.to_string().red().bold().to_string()
                    } else {
                        pane.to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        );
    }
}


mod backend_workers;
mod search_results;
mod ui;
use zellij_tile::prelude::*;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

use backend_workers::{FileContentsWorker, FileNameWorker, MessageToPlugin, MessageToSearch};
use search_results::{ResultsOfSearch, SearchResult};

pub const ROOT: &str = "/host";
pub const CURRENT_SEARCH_TERM: &str = "/data/current_search_term";

register_plugin!(State);
register_worker!(FileNameWorker, file_name_search_worker, FILE_NAME_WORKER);
register_worker!(
    FileContentsWorker,
    file_contents_search_worker,
    FILE_CONTENTS_WORKER
);

#[derive(Default)]
struct State {
    search_term: String,
    file_name_search_results: Vec<SearchResult>,
    file_contents_search_results: Vec<SearchResult>,
    loading: bool,
    loading_animation_offset: u8,
    should_open_floating: bool,
    search_filter: SearchType,
    display_rows: usize,
    display_columns: usize,
    displayed_search_results: (usize, Vec<SearchResult>), // usize is selected index
    kiosk_mode: bool, // in this mode, monocle only opens files on top of itself
}

impl ZellijPlugin for State {
    fn load(&mut self, config: BTreeMap<String, String>) {
        self.loading = true;
        self.kiosk_mode = config.get("kiosk").map(|k| k == "true").unwrap_or(false);
        if let Some(search_type) = config.get("search_filter") {
            match search_type.as_str() {
                "file_names" => self.search_filter = SearchType::Names,
                "file_contents" => self.search_filter = SearchType::Contents,
                "all" => self.search_filter = SearchType::NamesAndContents,
                _ => {}
            }
        }
        request_permission(&[
            PermissionType::OpenFiles,
            PermissionType::ChangeApplicationState,
            PermissionType::OpenTerminalsOrPlugins,
        ]);
        subscribe(&[
            EventType::Key,
            EventType::Mouse,
            EventType::CustomMessage,
            EventType::Timer,
        ]);
        post_message_to(PluginMessage::new_to_worker(
            "file_name_search",
            &serde_json::to_string(&MessageToSearch::ScanFolder).unwrap(),
            "",
        ));
        post_message_to(PluginMessage::new_to_worker(
            "file_contents_search",
            &serde_json::to_string(&MessageToSearch::ScanFolder).unwrap(),
            "",
        ));
        self.loading = true;
        set_timeout(0.5); // for displaying loading animation
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::Timer(_elapsed) => {
                if self.loading {
                    set_timeout(0.5);
                    self.progress_animation();
                    should_render = true;
                }
            }
            Event::CustomMessage(message, payload) => match serde_json::from_str(&message) {
                Ok(MessageToPlugin::UpdateFileNameSearchResults) => {
                    if let Ok(results_of_search) = serde_json::from_str::<ResultsOfSearch>(&payload)
                    {
                        self.update_file_name_search_results(results_of_search);
                        should_render = true;
                    }
                }
                Ok(MessageToPlugin::UpdateFileContentsSearchResults) => {
                    if let Ok(results_of_search) = serde_json::from_str::<ResultsOfSearch>(&payload)
                    {
                        self.update_file_contents_search_results(results_of_search);
                        should_render = true;
                    }
                }
                Ok(MessageToPlugin::DoneScanningFolder) => {
                    self.loading = false;
                    should_render = true;
                }
                Err(e) => eprintln!("Failed to deserialize custom message: {:?}", e),
            },
            Event::Key(key) => {
                self.handle_key(key);
                should_render = true;
            }
            _ => {
                eprintln!("Unknown event: {}", event.to_string());
            }
        }
        should_render
    }
    fn render(&mut self, rows: usize, cols: usize) {
        self.change_size(rows, cols);
        print!("{}", self);
    }
}

impl State {
    pub fn handle_key(&mut self, key: KeyWithModifier) {
        match key.bare_key {
            BareKey::Down => self.move_search_selection_down(),
            BareKey::Up => self.move_search_selection_up(),
            BareKey::Enter => self.open_search_result_in_editor(),
            BareKey::Tab => {
                self.open_search_result_in_terminal()
            }
            BareKey::Char('f') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                self.should_open_floating = !self.should_open_floating;
            }
            BareKey::Char('r') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                self.toggle_search_filter()
            }
            BareKey::Esc => {
                if !self.search_term.is_empty() {
                    self.clear_state();
                } else {
                    hide_self();
                }
            }
            BareKey::Char('c') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                if !self.search_term.is_empty() {
                    self.clear_state();
                } else {
                    hide_self();
                }
            }
            _ => self.append_to_search_term(key),
        }
    }
    pub fn update_file_name_search_results(&mut self, mut results_of_search: ResultsOfSearch) {
        if self.search_term == results_of_search.search_term {
            self.file_name_search_results = results_of_search.search_results.drain(..).collect();
            self.update_displayed_search_results();
        }
    }
    pub fn update_file_contents_search_results(&mut self, mut results_of_search: ResultsOfSearch) {
        if self.search_term == results_of_search.search_term {
            self.file_contents_search_results =
                results_of_search.search_results.drain(..).collect();
            self.update_displayed_search_results();
        }
    }
    pub fn change_size(&mut self, rows: usize, cols: usize) {
        self.display_rows = rows;
        self.display_columns = cols;
    }
    pub fn progress_animation(&mut self) {
        if self.loading_animation_offset == u8::MAX {
            self.loading_animation_offset = 0;
        } else {
            self.loading_animation_offset = self.loading_animation_offset.saturating_add(1);
        }
    }
    pub fn number_of_lines_in_displayed_search_results(&self) -> usize {
        self.displayed_search_results
            .1
            .iter()
            .map(|l| l.rendered_height())
            .sum()
    }
    fn move_search_selection_down(&mut self) {
        if self.displayed_search_results.0 < self.max_search_selection_index() {
            self.displayed_search_results.0 += 1;
        }
    }
    fn move_search_selection_up(&mut self) {
        self.displayed_search_results.0 = self.displayed_search_results.0.saturating_sub(1);
    }
    fn open_search_result_in_editor(&mut self) {
        match self.selected_search_result_entry() {
            Some(SearchResult::File { path, .. }) => {
                let ctx = BTreeMap::new();
                if self.kiosk_mode {
                    open_file_in_place(FileToOpen::new(PathBuf::from(path)), ctx)
                } else if self.should_open_floating {
                    open_file_floating(FileToOpen::new(PathBuf::from(path)), None, ctx)
                } else {
                    open_file(FileToOpen::new(PathBuf::from(path)), ctx);
                }
            }
            Some(SearchResult::LineInFile {
                path, line_number, ..
            }) => {
                let ctx = BTreeMap::new();
                if self.kiosk_mode {
                    open_file_in_place(
                        FileToOpen::new(PathBuf::from(path)).with_line_number(line_number),
                        ctx,
                    );
                } else if self.should_open_floating {
                    open_file_floating(
                        FileToOpen::new(PathBuf::from(path)).with_line_number(line_number),
                        None,
                        ctx,
                    );
                } else {
                    open_file(
                        FileToOpen::new(PathBuf::from(path)).with_line_number(line_number),
                        ctx,
                    );
                }
            }
            None => eprintln!("Search results not found"),
        }
    }
    fn open_search_result_in_terminal(&mut self) {
        let dir_path_of_result = |path: &str| -> PathBuf {
            let file_path = PathBuf::from(path);
            let mut dir_path = file_path.components();
            dir_path.next_back(); // remove file name to stay with just the folder
            dir_path.as_path().into()
        };
        let selected_search_result_entry = self.selected_search_result_entry();
        if let Some(SearchResult::File { path, .. }) | Some(SearchResult::LineInFile { path, .. }) =
            selected_search_result_entry
        {
            let dir_path = dir_path_of_result(&path);
            if self.kiosk_mode {
                open_terminal_in_place(&dir_path);
            } else if self.should_open_floating {
                open_terminal_floating(&dir_path, None);
            } else {
                open_terminal(&dir_path);
            }
        }
    }
    fn toggle_search_filter(&mut self) {
        self.search_filter.progress();
        self.send_search_query();
    }
    fn clear_state(&mut self) {
        self.file_name_search_results.clear();
        self.file_contents_search_results.clear();
        self.displayed_search_results = (0, vec![]);
        self.search_term.clear();
    }
    fn append_to_search_term(&mut self, key: KeyWithModifier) {
        match key.bare_key {
            BareKey::Char(character) => {
                self.search_term.push(character);
            }
            BareKey::Backspace => {
                self.search_term.pop();
                if self.search_term.len() == 0 {
                    self.clear_state();
                }
            }
            _ => {}
        }
        self.send_search_query();
    }
    fn send_search_query(&mut self) {
        match std::fs::write(CURRENT_SEARCH_TERM, &self.search_term) {
            Ok(_) => {
                if !self.search_term.is_empty() {
                    post_message_to(PluginMessage::new_to_worker(
                        "file_name_search",
                        &serde_json::to_string(&MessageToSearch::Search).unwrap(),
                        "",
                    ));
                    post_message_to(PluginMessage::new_to_worker(
                        "file_contents_search",
                        &serde_json::to_string(&MessageToSearch::Search).unwrap(),
                        "",
                    ));
                    self.file_name_search_results.clear();
                    self.file_contents_search_results.clear();
                }
            }
            Err(e) => eprintln!("Failed to write search term to HD, aborting search: {}", e),
        }
    }
    fn max_search_selection_index(&self) -> usize {
        self.displayed_search_results.1.len().saturating_sub(1)
    }
    fn update_displayed_search_results(&mut self) {
        if self.search_term.is_empty() {
            self.clear_state();
            return;
        }
        let mut search_results_of_interest = match self.search_filter {
            SearchType::NamesAndContents => {
                let mut all_search_results = self.file_name_search_results.clone();
                all_search_results.append(&mut self.file_contents_search_results.clone());
                all_search_results.sort_by(|a, b| b.score().cmp(&a.score()));
                all_search_results
            }
            SearchType::Names => self.file_name_search_results.clone(),
            SearchType::Contents => self.file_contents_search_results.clone(),
        };
        let mut height_taken_up_by_results = 0;
        let mut displayed_search_results = vec![];
        for search_result in search_results_of_interest.drain(..) {
            if height_taken_up_by_results + search_result.rendered_height()
                > self.rows_for_results()
            {
                break;
            }
            height_taken_up_by_results += search_result.rendered_height();
            displayed_search_results.push(search_result);
        }
        let new_index = self
            .selected_search_result_entry()
            .and_then(|currently_selected_search_result| {
                displayed_search_results
                    .iter()
                    .position(|r| r.is_same_entry(&currently_selected_search_result))
            })
            .unwrap_or(0);
        self.displayed_search_results = (new_index, displayed_search_results);
    }
    fn selected_search_result_entry(&self) -> Option<SearchResult> {
        self.displayed_search_results
            .1
            .get(self.displayed_search_results.0)
            .cloned()
    }
    pub fn rows_for_results(&self) -> usize {
        self.display_rows.saturating_sub(3) // search line and 2 controls lines
    }
}

#[derive(Serialize, Deserialize)]
pub enum SearchType {
    NamesAndContents,
    Names,
    Contents,
}

impl SearchType {
    pub fn progress(&mut self) {
        match &self {
            &SearchType::NamesAndContents => *self = SearchType::Names,
            &SearchType::Names => *self = SearchType::Contents,
            &SearchType::Contents => *self = SearchType::NamesAndContents,
        }
    }
}

impl Default for SearchType {
    fn default() -> Self {
        SearchType::NamesAndContents
    }
}


use zellij_tile::prelude::*;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ignore::Walk;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{self, BufRead};
use std::path::Path;
use unicode_width::UnicodeWidthStr;

use crate::search_results::{ResultsOfSearch, SearchResult};
use crate::{SearchType, CURRENT_SEARCH_TERM, ROOT};

static MAX_FILE_SIZE_BYTES: u64 = 1000000;

#[derive(Default, Serialize, Deserialize)]
pub struct Search {
    search_type: SearchType,
    file_names: BTreeSet<String>,
    file_contents: BTreeMap<(String, usize), String>, // file_name, line_number, line
    cached_file_name_results: HashMap<String, Vec<SearchResult>>,
    cached_file_contents_results: HashMap<String, Vec<SearchResult>>,
}

impl Search {
    pub fn new(search_type: SearchType) -> Self {
        Search {
            search_type,
            ..Default::default()
        }
    }
    fn on_message(&mut self, message: String, _payload: String) {
        match serde_json::from_str::<MessageToSearch>(&message) {
            Ok(MessageToSearch::ScanFolder) => {
                self.scan_hd();
                post_message_to_plugin(PluginMessage::new_to_plugin(
                    &serde_json::to_string(&MessageToPlugin::DoneScanningFolder).unwrap(),
                    "",
                ));
            }
            Ok(MessageToSearch::Search) => {
                if let Some(current_search_term) = self.read_search_term_from_hd_cache() {
                    self.search(current_search_term);
                }
            }
            Err(e) => eprintln!("Failed to deserialize worker message {:?}", e),
        }
    }
    pub fn scan_hd(&mut self) {
        for result in Walk::new(ROOT) {
            if let Ok(entry) = result {
                self.add_file_entry(entry.path(), entry.metadata().ok());
            }
        }
    }
    pub fn search(&mut self, search_term: String) {
        let search_results_limit = 100; // artificial limit to prevent probably unwanted chaos
        let mut file_names_search_results = None;
        let mut file_contents_search_results = None;
        if let SearchType::Names | SearchType::NamesAndContents = self.search_type {
            let file_names_matches = match self.cached_file_name_results.get(&search_term) {
                Some(cached_results) => cached_results.clone(),
                None => {
                    let mut matcher = SkimMatcherV2::default().use_cache(true);
                    let results = self.search_file_names(&search_term, &mut matcher);
                    self.cached_file_name_results
                        .insert(search_term.clone(), results.clone());
                    results
                }
            };
            file_names_search_results = Some(
                ResultsOfSearch::new(search_term.clone(), file_names_matches)
                    .limit_search_results(search_results_limit),
            );
        };
        if let SearchType::Contents | SearchType::NamesAndContents = self.search_type {
            let file_contents_matches = match self.cached_file_contents_results.get(&search_term) {
                Some(cached_results) => cached_results.clone(),
                None => {
                    let mut matcher = SkimMatcherV2::default().use_cache(true);
                    let results = self.search_file_contents(&search_term, &mut matcher);
                    self.cached_file_contents_results
                        .insert(search_term.clone(), results.clone());
                    results
                }
            };
            file_contents_search_results = Some(
                ResultsOfSearch::new(search_term.clone(), file_contents_matches)
                    .limit_search_results(search_results_limit),
            );
        };

        // if the search term changed before we finished, let's search again!
        if let Some(current_search_term) = self.read_search_term_from_hd_cache() {
            if current_search_term != search_term {
                return self.search(current_search_term.into());
            }
        }
        if let Some(file_names_search_results) = file_names_search_results {
            post_message_to_plugin(PluginMessage::new_to_plugin(
                &serde_json::to_string(&MessageToPlugin::UpdateFileNameSearchResults).unwrap(),
                &serde_json::to_string(&file_names_search_results).unwrap(),
            ));
        }
        if let Some(file_contents_search_results) = file_contents_search_results {
            post_message_to_plugin(PluginMessage::new_to_plugin(
                &serde_json::to_string(&MessageToPlugin::UpdateFileContentsSearchResults).unwrap(),
                &serde_json::to_string(&file_contents_search_results).unwrap(),
            ));
        }
    }
    fn add_file_entry(&mut self, file_name: &Path, file_metadata: Option<std::fs::Metadata>) {
        let file_path = file_name.display().to_string();
        let file_path_stripped_prefix = self.strip_file_prefix(&file_name);

        self.file_names.insert(file_path_stripped_prefix.clone());
        if let SearchType::NamesAndContents | SearchType::Contents = self.search_type {
            if file_metadata.map(|f| f.is_file()).unwrap_or(false) {
                if let Ok(file) = std::fs::File::open(&file_path) {
                    let file_size = file
                        .metadata()
                        .map(|f| f.len())
                        .unwrap_or(MAX_FILE_SIZE_BYTES);
                    if file_size >= MAX_FILE_SIZE_BYTES {
                        eprintln!(
                            "File {} too large, not indexing its contents",
                            file_name.display()
                        );
                        return;
                    }
                    let lines = io::BufReader::with_capacity(file_size as usize, file).lines();
                    for (index, line) in lines.enumerate() {
                        match line {
                            Ok(line) => {
                                self.file_contents.insert(
                                    (file_path_stripped_prefix.clone(), index + 1),
                                    String::from_utf8_lossy(&strip_ansi_escapes::strip(line))
                                        .to_string(),
                                );
                            }
                            Err(_) => {
                                break; // probably a binary file, skip it
                            }
                        }
                    }
                }
            }
        }
    }
    fn search_file_names(
        &self,
        search_term: &str,
        matcher: &mut SkimMatcherV2,
    ) -> Vec<SearchResult> {
        let mut matches = vec![];
        for entry in &self.file_names {
            if let Some((score, indices)) = matcher.fuzzy_indices(&entry, &search_term) {
                matches.push(SearchResult::new_file_name(
                    score,
                    indices,
                    entry.to_owned(),
                ));
            }
        }
        matches
    }
    fn search_file_contents(
        &self,
        search_term: &str,
        matcher: &mut SkimMatcherV2,
    ) -> Vec<SearchResult> {
        let mut matches = vec![];
        for ((file_name, line_number), line_entry) in &self.file_contents {
            if let Some((score, indices)) = matcher.fuzzy_indices(&line_entry, &search_term) {
                matches.push(SearchResult::new_file_line(
                    score,
                    indices,
                    file_name.clone(),
                    line_entry.clone(),
                    *line_number,
                ));
            }
        }
        matches
    }
    fn strip_file_prefix(&self, file_name: &Path) -> String {
        let mut file_path_stripped_prefix = file_name.display().to_string().split_off(ROOT.width());
        if file_path_stripped_prefix.starts_with('/') {
            file_path_stripped_prefix.remove(0);
        }
        file_path_stripped_prefix
    }
    fn read_search_term_from_hd_cache(&self) -> Option<String> {
        match std::fs::read(CURRENT_SEARCH_TERM) {
            Ok(current_search_term) => {
                Some(String::from_utf8_lossy(&current_search_term).to_string())
            }
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum MessageToSearch {
    ScanFolder,
    Search,
}

#[derive(Serialize, Deserialize)]
pub enum MessageToPlugin {
    UpdateFileNameSearchResults,
    UpdateFileContentsSearchResults,
    DoneScanningFolder,
}

#[derive(Serialize, Deserialize)]
pub struct FileNameWorker {
    search: Search,
}

impl Default for FileNameWorker {
    fn default() -> Self {
        FileNameWorker {
            search: Search::new(SearchType::Names),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct FileContentsWorker {
    search: Search,
}

impl Default for FileContentsWorker {
    fn default() -> Self {
        FileContentsWorker {
            search: Search::new(SearchType::Contents),
        }
    }
}

impl<'de> ZellijWorker<'de> for FileNameWorker {
    fn on_message(&mut self, message: String, payload: String) {
        self.search.on_message(message, payload);
    }
}

impl<'de> ZellijWorker<'de> for FileContentsWorker {
    fn on_message(&mut self, message: String, payload: String) {
        self.search.on_message(message, payload);
    }
}


