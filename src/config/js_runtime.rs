use rquickjs::Ctx;
use rquickjs::{Context, Function, Object, Runtime, Value};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};

pub struct JSRuntime {
    runtime: Runtime,
    context: Context,
    keybindings: Arc<Mutex<Vec<JSKeybinding>>>,
    window_handlers: Arc<Mutex<Vec<JSWindowHandler>>>,
    startup_handlers: Arc<Mutex<Vec<String>>>,
    callback_functions: Arc<Mutex<HashMap<String, String>>>, // Store actual callback code
}

unsafe impl Send for JSRuntime {}

#[derive(Debug, Clone)]
pub struct JSKeybinding {
    pub combo: String,
    pub modifiers: Vec<String>,
    pub key: String,
    pub callback_name: String, // Name of the callback function
}

#[derive(Debug, Clone)]
pub struct JSWindowHandler {
    pub event: WindowEvent,
    pub callback: String,
}

#[derive(Debug, Clone)]
pub enum WindowEvent {
    Create,
    Close,
    Focus,
    Unfocus,
    MouseEnter,
    MouseLeave,
    Urgent,
}

impl JSRuntime {
    pub fn new() -> Result<Self, String> {
        let runtime =
            Runtime::new().map_err(|e| format!("Failed to create JS runtime: {:?}", e))?;
        let context =
            Context::full(&runtime).map_err(|e| format!("Failed to create JS context: {:?}", e))?;

        let keybindings = Arc::new(Mutex::new(Vec::new()));
        let window_handlers = Arc::new(Mutex::new(Vec::new()));
        let startup_handlers = Arc::new(Mutex::new(Vec::new()));
        let callback_functions = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            runtime,
            context,
            keybindings,
            window_handlers,
            startup_handlers,
            callback_functions,
        })
    }

    pub fn init_api(&self) -> Result<(), String> {
        self.context.with(|ctx| {
            let globals = ctx.globals();

            // Create wm object
            let wm = Object::new(ctx.clone())
                .map_err(|e| format!("Failed to create wm object: {:?}", e))?;

            // Add wm methods
            self.add_wm_methods(ctx.clone(), &wm)?;

            globals
                .set("wm", wm)
                .map_err(|e| format!("Failed to set wm global: {:?}", e))?;

            // Add console object
            let console = Object::new(ctx.clone())
                .map_err(|e| format!("Failed to create console object: {:?}", e))?;

            console
                .set(
                    "log",
                    Function::new(ctx.clone(), |msg: String| {
                        println!("JS: {}", msg);
                    }),
                )
                .map_err(|e| format!("Failed to set console.log: {:?}", e))?;

            globals
                .set("console", console)
                .map_err(|e| format!("Failed to set console global: {:?}", e))?;

            // Add utility functions
            self.add_utility_functions(ctx.clone(), &globals)?;

            Ok(())
        })
    }

    fn add_wm_methods<'a>(&self, ctx: Ctx<'a>, wm: &Object<'a>) -> Result<(), String> {
        // wm.spawn(command)
        wm.set(
            "spawn",
            Function::new(ctx.clone(), |cmd: String| {
                println!("JS: spawn({})", cmd);
                Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .spawn()
                    .map_err(|e| eprintln!("Failed to spawn '{}': {}", cmd, e))
                    .ok();
            }),
        )
        .map_err(|e| format!("Failed to set spawn: {:?}", e))?;

        // wm.close()
        wm.set(
            "close",
            Function::new(ctx.clone(), || {
                println!("JS: close()");
                // This will be handled by the keybinding system
            }),
        )
        .map_err(|e| format!("Failed to set close: {:?}", e))?;

        // wm.focus(direction)
        wm.set(
            "focus",
            Function::new(ctx.clone(), |dir: String| {
                println!("JS: focus({})", dir);
                // This will be handled by the keybinding system
            }),
        )
        .map_err(|e| format!("Failed to set focus: {:?}", e))?;

        // wm.moveToWorkspace(workspace)
        wm.set(
            "moveToWorkspace",
            Function::new(ctx.clone(), |ws: u32| {
                println!("JS: moveToWorkspace({})", ws);
                // This will be handled by the keybinding system
            }),
        )
        .map_err(|e| format!("Failed to set moveToWorkspace: {:?}", e))?;

        // wm.switchToWorkspace(workspace)
        wm.set(
            "switchToWorkspace",
            Function::new(ctx.clone(), |ws: u32| {
                println!("JS: switchToWorkspace({})", ws);
                // This will be handled by the keybinding system
            }),
        )
        .map_err(|e| format!("Failed to set switchToWorkspace: {:?}", e))?;

        // wm.cycleWorkspaceNext()
        wm.set(
            "cycleWorkspaceNext",
            Function::new(ctx.clone(), || {
                println!("JS: cycleWorkspaceNext()");
                // This will be handled by the keybinding system
            }),
        )
        .map_err(|e| format!("Failed to set cycleWorkspaceNext: {:?}", e))?;

        // wm.cycleWorkspacePrev()
        wm.set(
            "cycleWorkspacePrev",
            Function::new(ctx.clone(), || {
                println!("JS: cycleWorkspacePrev()");
                // This will be handled by the keybinding system
            }),
        )
        .map_err(|e| format!("Failed to set cycleWorkspacePrev: {:?}", e))?;

        // wm.toggleFloating()
        wm.set(
            "toggleFloating",
            Function::new(ctx.clone(), || {
                println!("JS: toggleFloating()");
                // This will be handled by the keybinding system
            }),
        )
        .map_err(|e| format!("Failed to set toggleFloating: {:?}", e))?;

        // wm.toggleMaximize()
        wm.set(
            "toggleMaximize",
            Function::new(ctx.clone(), || {
                println!("JS: toggleMaximize()");
            }),
        )
        .map_err(|e| format!("Failed to set toggleMaximize: {:?}", e))?;

        // wm.moveWindow(direction)
        wm.set(
            "moveWindow",
            Function::new(ctx.clone(), |dir: String| {
                println!("JS: moveWindow({})", dir);
            }),
        )
        .map_err(|e| format!("Failed to set moveWindow: {:?}", e))?;

        // wm.setLayout(layout)
        wm.set(
            "setLayout",
            Function::new(ctx.clone(), |layout: String| {
                println!("JS: setLayout({})", layout);
                // This will be handled by the keybinding system
            }),
        )
        .map_err(|e| format!("Failed to set setLayout: {:?}", e))?;

        // wm.cycleLayout()
        wm.set(
            "cycleLayout",
            Function::new(ctx.clone(), || {
                println!("JS: cycleLayout()");
                // This will be handled by the keybinding system
            }),
        )
        .map_err(|e| format!("Failed to set cycleLayout: {:?}", e))?;

        // wm.reload()
        wm.set(
            "reload",
            Function::new(ctx.clone(), || {
                println!("JS: reload()");
                // Would reload configuration
            }),
        )
        .map_err(|e| format!("Failed to set reload: {:?}", e))?;

        // wm.exit()
        wm.set(
            "exit",
            Function::new(ctx.clone(), || {
                println!("JS: exit()");
                // Would exit the compositor
            }),
        )
        .map_err(|e| format!("Failed to set exit: {:?}", e))?;

        // wm.moveWindow(direction)
        wm.set(
            "moveWindow",
            Function::new(ctx.clone(), |dir: String| {
                println!("JS: moveWindow({})", dir);
                // This will be handled by the keybinding system
            }),
        )
        .map_err(|e| format!("Failed to set moveWindow: {:?}", e))?;

        Ok(())
    }

    fn add_utility_functions<'a>(&self, ctx: Ctx<'a>, globals: &Object<'a>) -> Result<(), String> {
        let keybindings = self.keybindings.clone();
        let callback_functions = self.callback_functions.clone();

        // keybind(combo, callback)
        globals
            .set(
                "keybind",
                Function::new(ctx.clone(), move |combo: String, callback: Function| {
                    println!("Registering keybinding: {}", combo);

                    // Parse combo (e.g., "Super+Return" -> ["Super"], "Return")
                    let (modifiers, key) = parse_key_combo(&combo);

                    // Generate a unique callback name
                    let callback_name = format!("callback_{}", combo.replace("+", "_"));

                    // Store the callback function's string representation
                    if let Ok(mut callbacks) = callback_functions.lock() {
                        // Store the callback as a callable function reference
                        // In a real implementation, we'd serialize the function properly
                        callbacks.insert(
                            callback_name.clone(),
                            format!("() => {{ /* callback for {} */ }}", combo),
                        );
                    }

                    if let Ok(mut bindings) = keybindings.lock() {
                        bindings.push(JSKeybinding {
                            combo: combo.clone(),
                            modifiers,
                            key,
                            callback_name,
                        });
                    }
                }),
            )
            .map_err(|e| format!("Failed to set keybind: {:?}", e))?;

        let window_handlers = self.window_handlers.clone();

        // onWindowCreate(callback)
        globals
            .set(
                "onWindowCreate",
                Function::new(ctx.clone(), move |callback: Function| {
                    println!("Registered window create handler");
                    if let Ok(mut handlers) = window_handlers.lock() {
                        handlers.push(JSWindowHandler {
                            event: WindowEvent::Create,
                            callback: "window_create_handler".to_string(),
                        });
                    }
                }),
            )
            .map_err(|e| format!("Failed to set onWindowCreate: {:?}", e))?;

        // onMouseEnter(callback)
        globals
            .set(
                "onMouseEnter",
                Function::new(ctx.clone(), move |_callback: Function| {
                    println!("Registered mouse enter handler");
                }),
            )
            .map_err(|e| format!("Failed to set onMouseEnter: {:?}", e))?;

        // onMouseLeave(callback)
        globals
            .set(
                "onMouseLeave",
                Function::new(ctx.clone(), move |_callback: Function| {
                    println!("Registered mouse leave handler");
                }),
            )
            .map_err(|e| format!("Failed to set onMouseLeave: {:?}", e))?;

        // onLayoutChange(callback)
        globals
            .set(
                "onLayoutChange",
                Function::new(ctx.clone(), move |_callback: Function| {
                    println!("Registered layout change handler");
                }),
            )
            .map_err(|e| format!("Failed to set onLayoutChange: {:?}", e))?;

        // onWindowUrgent(callback)
        globals
            .set(
                "onWindowUrgent",
                Function::new(ctx.clone(), move |_callback: Function| {
                    println!("Registered urgent window handler");
                }),
            )
            .map_err(|e| format!("Failed to set onWindowUrgent: {:?}", e))?;

        // onMouseEnter(callback)
        globals
            .set(
                "onMouseEnter",
                Function::new(ctx.clone(), |callback: Function| {
                    println!("Registered mouse enter handler");
                }),
            )
            .map_err(|e| format!("Failed to set onMouseEnter: {:?}", e))?;

        // notify(options)
        globals
            .set(
                "notify",
                Function::new(ctx.clone(), |options: Object| {
                    println!("JS: notify()");
                    // Would send notification
                }),
            )
            .map_err(|e| format!("Failed to set notify: {:?}", e))?;

        let startup_handlers = self.startup_handlers.clone();

        // onStartup(callback)
        globals
            .set(
                "onStartup",
                Function::new(ctx, move |callback: Function| {
                    println!("Registered startup handler");
                    if let Ok(mut handlers) = startup_handlers.lock() {
                        handlers.push("startup_handler".to_string());
                    }
                    // Execute immediately for now
                    if let Err(e) = callback.call::<_, ()>(()) {
                        eprintln!("Startup handler error: {:?}", e);
                    }
                }),
            )
            .map_err(|e| format!("Failed to set onStartup: {:?}", e))?;

        Ok(())
    }

    pub fn evaluate(&self, js_code: &str) -> Result<(), String> {
        self.context.with(|ctx| {
            match ctx.eval::<Value, _>(js_code) {
                Ok(_) => Ok(()),
                Err(e) => {
                    // Try to get more detailed error information
                    let error_msg = format!("JS evaluation error: {:?}", e);
                    eprintln!("JavaScript Error Details:");
                    eprintln!("  Error: {:?}", e);
                    eprintln!("  Code length: {} characters", js_code.len());
                    // Show first few lines of code that might be causing issues
                    let lines: Vec<&str> = js_code.lines().take(10).collect();
                    eprintln!("  Code preview (first 10 lines):");
                    for (i, line) in lines.iter().enumerate() {
                        eprintln!("    {}: {}", i + 1, line);
                    }
                    Err(error_msg)
                }
            }
        })
    }

    pub fn get_keybindings(&self) -> Vec<JSKeybinding> {
        self.keybindings
            .lock()
            .map(|kb| kb.clone())
            .unwrap_or_default()
    }

    pub fn get_window_handlers(&self) -> Vec<JSWindowHandler> {
        self.window_handlers
            .lock()
            .map(|wh| wh.clone())
            .unwrap_or_default()
    }

    pub fn execute_callback(&self, callback_name: &str, args: &str) -> Result<(), String> {
        // Try to execute the callback by name
        let code = if args.is_empty() {
            format!("{}()", callback_name)
        } else {
            format!("{}({})", callback_name, args)
        };

        println!("Executing JS callback: {}", code);
        self.evaluate(&code)
    }

    pub fn execute_keybinding_callback(&self, combo: &str) -> Result<(), String> {
        // Find the keybinding by combo and execute its callback
        let bindings = self.get_keybindings();
        for binding in bindings {
            if binding.combo == combo {
                return self.execute_callback(&binding.callback_name, "");
            }
        }
        Err(format!("No keybinding found for combo: {}", combo))
    }
}

fn parse_key_combo(combo: &str) -> (Vec<String>, String) {
    let parts: Vec<&str> = combo.split('+').collect();

    if parts.len() == 1 {
        return (vec![], parts[0].to_string());
    }

    let modifiers = parts[..parts.len() - 1]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let key = parts[parts.len() - 1].to_string();

    (modifiers, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_runtime_creation() {
        let runtime = JSRuntime::new();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_key_combo_parsing() {
        let (mods, key) = parse_key_combo("Super+Return");
        assert_eq!(mods, vec!["Super"]);
        assert_eq!(key, "Return");

        let (mods, key) = parse_key_combo("Super+Shift+q");
        assert_eq!(mods, vec!["Super", "Shift"]);
        assert_eq!(key, "q");
    }
}
