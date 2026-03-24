use crate::kernel::unit_of_work::UnitOfWork;

/// Try to handle as a builtin. Returns Some(exit_code) if handled.
pub fn try_handle(args: &[String], uow: &mut UnitOfWork, history: &[String]) -> Option<i32> {
    let cmd = args.first()?.as_str();
    match cmd {
        "cd" => {
            let target = args.get(1).map(|s| s.as_str());
            let new_path = uow.session.resolve_path(target);
            uow.session.set_cwd(new_path);
            Some(0)
        }
        "pwd" => {
            let cwd = uow.session.cwd().to_string();
            uow.print(cwd);
            Some(0)
        }
        "exit" | "logout" => {
            uow.request_exit();
            Some(0)
        }
        "history" => {
            for (i, h) in history.iter().enumerate() {
                uow.print(format!("{:4}  {h}", i + 1));
            }
            Some(0)
        }
        "true" => Some(0),
        "false" => Some(1),
        _ => None,
    }
}
