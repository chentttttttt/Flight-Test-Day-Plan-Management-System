use pyo3::prelude::*;
use pyo3::types::PyModule;
use serde_json::Value;
use std::error::Error;
use std::fs::metadata;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::SystemTime;
use std::sync::Mutex;

/// 缓存模块及其最后修改时间
static MODULE_CACHE: OnceLock<Mutex<Option<(Py<PyModule>, SystemTime)>>> = OnceLock::new();

/// 获取 solver 模块（自动重载）
fn get_solver_module(py: Python<'_>) -> Result<Py<PyModule>, Box<dyn Error>> {
    let cache = MODULE_CACHE.get_or_init(|| Mutex::new(None));
    let mut guard = cache.lock().unwrap();

    let solver_path = PathBuf::from("solver.py");

    // 检查文件是否存在
    if !solver_path.exists() {
        return Err(format!("solver.py not found in current directory: {:?}", std::env::current_dir()?).into());
    }

    let current_mtime = metadata(&solver_path)?.modified()?;

    // 判断是否需要重载（文件修改时间变化 或 首次加载）
    let should_reload = match guard.as_ref() {
        Some((_, mtime)) if *mtime == current_mtime => false,
        _ => true,
    };

    if should_reload {
        println!("[AutoReload] Detected change in solver.py, reloading module...");

        // 确保项目目录和虚拟环境目录在 sys.path 中（保持原功能）
        let syspath = py.import("sys")?.getattr("path")?;
        let project_dir = std::env::current_dir()?;
        syspath.call_method1("insert", (0, project_dir.to_str().unwrap()))?;
        // 虚拟环境路径
        let site_packages = std::env::current_dir()?
            .join(".venv")
            .join("Lib")
            .join("site-packages")
            .to_str()
            .ok_or("Invalid path")?
            .to_string();
        syspath.call_method1("insert", (0, site_packages))?;
        println!("sys.path: {:?}", syspath);  // 保持原有打印

        // 重载或首次导入
        let module = if let Some((ref old_module, _)) = *guard {
            let importlib = py.import("importlib")?;
            importlib.call_method1("reload", (old_module.as_ref(py),))?;
            println!("[AutoReload] Module reloaded successfully.");
            old_module.clone()
        } else {
            let module = py.import("solver")?.into();
            println!("[AutoReload] Module loaded for the first time.");
            module
        };

        *guard = Some((module.clone(), current_mtime));
        Ok(module)
    } else {
        Ok(guard.as_ref().unwrap().0.clone())
    }
}

/// 调用 Python MILP 求解器（支持 solver.py 热重载）
pub fn solve_plan(input: &Value) -> Result<Value, Box<dyn Error>> {
    Python::with_gil(|py| {
        let module = get_solver_module(py)?;
        let solver = module.as_ref(py);
        let input_str = serde_json::to_string(input)?;
        let result_str: String = solver.call_method("solve_from_json", (input_str,), None)?.extract()?;
        let result: Value = serde_json::from_str(&result_str)?;
        println!("{:#?}", result);  // 保持原有打印
        Ok(result)
    })
}