use rquickjs::{Context, Runtime, Function, Object, Value};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct JSRuntime {
    runtime: Runtime,
    context: Context,
    keybindings: Arc<Mutex<Vec<JSKeybinding>>>,
    window_handlers: Arc<Mutex<Vec<JSWindowHandler>>>,
    startup_handlers: Arc<Mutex<Vec<String>>>,
}

#[derive(Debug, Clone)]
pub struct JSKeybinding {
    pub combo: String,
    pub modifiers: Vec<String>,
    pub key: String,
    pub callback: String, // JavaScript code to execute
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
        let runtime = Runtime::new()
            .map_err(|e| format!("Failed to create JS runtime: {:?}", e))?;
        let context = Context::full(&runtime)
            .map_err(|e| format!("Failed to create JS context: {:?}", e))?;
        
        let keybindings = Arc::new(Mutex::new(Vec::new()));
        let window_handlers = Arc::new(Mutex::new(Vec::new()));
        let startup_handlers = Arc::new(Mutex::new(Vec::new()));
        
        Ok(Self {
            runtime,
            context,
            keybindings,
            window_handlers,
            startup_handlers,
        })
    }
    
    pub fn init_api(&self) -> Result<(), String> {
        self.context.with(|ctx| {
            let globals = ctx.globals();
            
            // Create wm object
            let wm = Object::new(ctx.clone())
                .map_err(|e| format!("Failed to create wm object: {:?}", e))?;
            
            // Add wm methods
            self.add_wm_methods(&wm)?;
            
            globals.set("wm", wm)
                .map_err(|e| format!("Failed to set wm global: {:?}", e))?;
            
            // Add utility functions
            self.add_utility_functions(&globals)?;
            
            // wm.switchToWorkspace(workspace)
        wm.set("switchToWorkspace", Function::new(
            self.context.clone(),
            |ws: u32| {
                println!("JS: switchToWorkspace({})", ws);
            }
        )).map_err(|e| format!("Failed to set switchToWorkspace: {:?}", e))?;
        
        // wm.cycleWorkspaceNext()
        wm.set("cycleWorkspaceNext", Function::new(
            self.context.clone(),
            || {
                println!("JS: cycleWorkspaceNext()");
            }
        )).map_err(|e| format!("Failed to set cycleWorkspaceNext: {:?}", e))?;
        
        // wm.cycleWorkspacePrev()
        wm.set("cycleWorkspacePrev", Function::new(
            self.context.clone(),
            || {
                println!("JS: cycleWorkspacePrev()");
            }
        )).map_err(|e| format!("Failed to set cycleWorkspacePrev: {:?}", e))?;
        
        Ok(())
        })
    }
    
    fn add_wm_methods(&self, wm: &Object) -> Result<(), String> {
        // These would be actual implementations
        // For now, we'll store them as callbacks
        
        // wm.spawn(command)
        wm.set("spawn", Function::new(
            self.context.clone(),
            |cmd: String| {
                println!("JS: spawn({})", cmd);
                // Would actually spawn process
            }
        )).map_err(|e| format!("Failed to set spawn: {:?}", e))?;
        
        // wm.close()
        wm.set("close", Function::new(
            self.context.clone(),
            || {
                println!("JS: close()");
                // Would close focused window
            }
        )).map_err(|e| format!("Failed to set close: {:?}", e))?;
        
        // wm.focus(direction)
        wm.set("focus", Function::new(
            self.context.clone(),
            |dir: String| {
                println!("JS: focus({})", dir);
                // Would focus in direction
            }
        )).map_err(|e| format!("Failed to set focus: {:?}", e))?;
        
        // wm.moveToWorkspace(workspace)
        wm.set("moveToWorkspace", Function::new(
            self.context.clone(),
            |ws: u32| {
                println!("JS: moveToWorkspace({})", ws);
            }
        )).map_err(|e| format!("Failed to set moveToWorkspace: {:?}", e))?;
        
        // wm.switchToWorkspace(workspace)
        wm.set("switchToWorkspace", Function::new(
            self.context.clone(),
            |ws: u32| {
                println!("JS: switchToWorkspace({})", ws);
            }
        )).map_err(|e| format!("Failed to set switchToWorkspace: {:?}", e))?;
        
        // wm.toggleFloating()
        wm.set("toggleFloating", Function::new(
            self.context.clone(),
            || {
                println!("JS: toggleFloating()");
            }
        )).map_err(|e| format!("Failed to set toggleFloating: {:?}", e))?;
        
        Ok(())
    }
    
    fn add_utility_functions(&self, globals: &Object) -> Result<(), String> {
        let keybindings = self.keybindings.clone();
        
        // keybind(combo, callback)
        globals.set("keybind", Function::new(
            self.context.clone(),
            move |combo: String, callback: Function| {
                println!("Registering keybinding: {}", combo);
                
                // Parse combo (e.g., "Super+Return" -> ["Super"], "Return")
                let (modifiers, key) = parse_key_combo(&combo);
                
                // Store the callback as a string (would serialize function)
                let callback_str = format!("callback_{}", combo);
                
                if let Ok(mut bindings) = keybindings.lock() {
                    bindings.push(JSKeybinding {
                        combo: combo.clone(),
                        modifiers,
                        key,
                        callback: callback_str,
                    });
                }
            }
        )).map_err(|e| format!("Failed to set keybind: {:?}", e))?;
        
        let window_handlers = self.window_handlers.clone();
        
        // onWindowCreate(callback)
        globals.set("onWindowCreate", Function::new(
            self.context.clone(),
            move |callback: Function| {
                println!("Registered window create handler");
                if let Ok(mut handlers) = window_handlers.lock() {
                    handlers.push(JSWindowHandler {
                        event: WindowEvent::Create,
                        callback: "window_create_handler".to_string(),
                    });
                }
            }
        )).map_err(|e| format!("Failed to set onWindowCreate: {:?}", e))?;
        
        // onMouseEnter(callback)
        globals.set("onMouseEnter", Function::new(
            self.context.clone(),
            |callback: Function| {
                println!("Registered mouse enter handler");
            }
        )).map_err(|e| format!("Failed to set onMouseEnter: {:?}", e))?;
        
        // notify(options)
        globals.set("notify", Function::new(
            self.context.clone(),
            |options: Object| {
                println!("JS: notify()");
                // Would send notification
            }
        )).map_err(|e| format!("Failed to set notify: {:?}", e))?;
        
        let startup_handlers = self.startup_handlers.clone();
        
        // onStartup(callback)
        globals.set("onStartup", Function::new(
            self.context.clone(),
            move |callback: Function| {
                println!("Registered startup handler");
                if let Ok(mut handlers) = startup_handlers.lock() {
                    handlers.push("startup_handler".to_string());
                }
                // Execute immediately for now
                if let Err(e) = callback.call::<_, ()>(()) {
                    eprintln!("Startup handler error: {:?}", e);
                }
            }
        )).map_err(|e| format!("Failed to set onStartup: {:?}", e))?;
        
        Ok(())
    }
    
    pub fn evaluate(&self, js_code: &str) -> Result<(), String> {
        self.context.with(|ctx| {
            ctx.eval::<Value, _>(js_code)
                .map_err(|e| format!("JS evaluation error: {:?}", e))?;
            Ok(())
        })
    }
    
    pub fn get_keybindings(&self) -> Vec<JSKeybinding> {
        self.keybindings.lock()
            .map(|kb| kb.clone())
            .unwrap_or_default()
    }
    
    pub fn get_window_handlers(&self) -> Vec<JSWindowHandler> {
        self.window_handlers.lock()
            .map(|wh| wh.clone())
            .unwrap_or_default()
    }
    
    pub fn execute_callback(&self, callback_name: &str, args: &str) -> Result<(), String> {
        let code = format!("{}({})", callback_name, args);
        self.evaluate(&code)
    }
}

fn parse_key_combo(combo: &str) -> (Vec<String>, String) {
    let parts: Vec<&str> = combo.split('+').collect();
    
    if parts.len() == 1 {
        return (vec![], parts[0].to_string());
    }
    
    let modifiers = parts[..parts.len()-1]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let key = parts[parts.len()-1].to_string();
    
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
