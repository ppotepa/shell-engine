// Scripted terminal demo — all commands handled in Rhai script.

// Initialize command counter in state
if !state.has("command_count") {
  state["command_count"] = 0;
}

// Handle submitted commands
if ui.has_submit && ui.focused_target == "ui-terminal-prompt" {
  let cmd = ui.submit_text.trim();
  
  if !cmd.is_empty() {
    state["command_count"] += 1;
    let count = state["command_count"];
    
    // Parse command
    let parts = cmd.split();
    let command = if parts.len() > 0 { parts[0] } else { "" };
    
    // Handle commands via terminal API
    if command == "help" {
      terminal.push("Available commands:");
      terminal.push("  help    — show this message");
      terminal.push("  status  — show system status");
      terminal.push("  count   — show command count");
      terminal.push("  clear   — clear terminal");
      terminal.push("  echo    — echo arguments");
    } else if command == "status" {
      terminal.push("power: online");
      terminal.push("hull: 92%");
      terminal.push("signal: stable");
      terminal.push("commands executed: " + count);
    } else if command == "count" {
      terminal.push("Commands executed: " + count);
    } else if command == "clear" {
      terminal.clear();
    } else if command == "echo" {
      let args = parts.extract(1, parts.len());
      if args.len() > 0 {
        terminal.push(args.join(" "));
      }
    } else {
      terminal.push("unknown command: " + command);
      terminal.push("type 'help' for available commands");
    }
  }
}
