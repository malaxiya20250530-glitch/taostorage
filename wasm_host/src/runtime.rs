use anyhow::Context;
use wasmi::{Engine, Module, Store, Linker};

/// WASM 运行时封装 — "长生久视"的演化引擎
///
/// 使用 wasmi（纯 Rust WASM 解释器），轻量、无 JIT 依赖，
/// 契合"和其光"的普适性：可嵌入任何设备。
pub struct WasmQiRuntime {
    engine: Engine,
    /// 缓存的模块（用于热重载）
    wasm_bytes: Vec<u8>,
}

impl WasmQiRuntime {
    /// 从 WASM 字节码初始化运行时
    pub fn new(wasm_bytes: &[u8]) -> anyhow::Result<Self> {
        let engine = Engine::default();

        // 预编译验证
        let _module = Module::new(&engine, wasm_bytes)
            .context("invalid WASM module")?;

        Ok(Self {
            engine,
            wasm_bytes: wasm_bytes.to_vec(),
        })
    }

    /// 热重载：替换博弈规则
    pub fn reload(&mut self, wasm_bytes: &[u8]) -> anyhow::Result<()> {
        let _module = Module::new(&self.engine, wasm_bytes)
            .context("invalid WASM module for reload")?;
        self.wasm_bytes = wasm_bytes.to_vec();
        Ok(())
    }

    /// 调用导出的 qi 决策函数
    ///
    /// 函数签名（WASM 侧）：
    ///   fn execute_qi(hexagram: i32, replicas: i32, target: i32) -> i32
    ///
    /// 返回值映射到 QiAction：
    ///   0 = Noop, 1 = Replicate, 2 = EmergencyRebuild,
    ///   3 = PromoteToHot, 4 = DemoteToCold, 5 = Archive, 6 = Stabilized
    pub fn execute_qi(
        &mut self,
        hexagram: i32,
        replicas: i32,
        target: i32,
    ) -> anyhow::Result<i32> {
        let module = Module::new(&self.engine, &self.wasm_bytes)?;

        let linker = Linker::new(&self.engine);
        let mut store = Store::new(&self.engine, ());

        let instance = linker
            .instantiate(&mut store, &module)?
            .start(&mut store)?;

        let func = instance
            .get_typed_func::<(i32, i32, i32), i32>(&store, "execute_qi")
            .context("WASM module missing 'execute_qi' export")?;

        let result = func.call(&mut store, (hexagram, replicas, target))?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造最小 WASM 模块用于测试
    /// 等价 WAT:
    ///   (module
    ///     (func (export "execute_qi") (param i32 i32 i32) (result i32)
    ///       local.get 1
    ///       local.get 2
    ///       i32.lt_s
    ///       (if (result i32) (then i32.const 1) (else i32.const 0))))
    fn minimal_wasm_qi() -> Vec<u8> {
        let mut wasm = Vec::new();

        // magic number
        wasm.extend_from_slice(&[0x00, 0x61, 0x73, 0x6d]);
        // version
        wasm.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);

        // type section: 1 type: (i32, i32, i32) -> i32
        wasm.push(0x01); // section id
        wasm.push(0x08); // section size
        wasm.push(0x01); // count
        wasm.push(0x60); // functype
        wasm.push(0x03); // 3 params
        wasm.push(0x7f); // i32
        wasm.push(0x7f); // i32
        wasm.push(0x7f); // i32
        wasm.push(0x01); // 1 result
        wasm.push(0x7f); // i32

        // function section: 1 function, type index 0
        wasm.push(0x03); // section id
        wasm.push(0x02); // section size
        wasm.push(0x01); // count
        wasm.push(0x00); // type index

        // export section: "execute_qi" -> func 0
        let name = b"execute_qi";
        wasm.push(0x07); // section id
        wasm.push(0x0e); // section size: 1 + (1 + 10) + 1 + 1 = 14
        wasm.push(0x01); // count
        wasm.push(0x0a); // name length
        wasm.extend_from_slice(name);
        wasm.push(0x00); // export kind: function
        wasm.push(0x00); // function index

        // code section
        // body: 0 locals, then:
        //   local.get 1 (0x20 0x01)
        //   local.get 2 (0x20 0x02)
        //   i32.lt_s   (0x48)
        //   if (result i32) 0x7f (0x04 0x7f)
        //     i32.const 1 (0x41 0x01)
        //   else            (0x05)
        //     i32.const 0 (0x41 0x00)
        //   end              (0x0b)
        // end                 (0x0b)
        // body size = 1 (locals) + 14 (instrs) + 1 (end) = 16 = 0x10
        wasm.push(0x0a); // section id
        wasm.push(0x11); // section size: 1 + 1 + 15 = 17
        wasm.push(0x01); // count
        wasm.push(0x0f); // body size
        wasm.push(0x00); // 0 locals
        // instructions
        wasm.extend_from_slice(&[
            0x20, 0x01, // local.get 1
            0x20, 0x02, // local.get 2
            0x48,       // i32.lt_s
            0x04, 0x7f, // if (result i32)
            0x41, 0x01, // i32.const 1
            0x05,       // else
            0x41, 0x00, // i32.const 0
            0x0b,       // end if
            0x0b,       // end function
        ]);

        wasm
    }

    #[test]
    fn test_wasm_qi_replicate_when_insufficient() {
        let wasm = minimal_wasm_qi();
        let mut rt = WasmQiRuntime::new(&wasm).expect("valid wasm");
        // replicas=1, target=3 → should replicate (1)
        let result = rt.execute_qi(0, 1, 3).expect("exec");
        assert_eq!(result, 1);
    }

    #[test]
    fn test_wasm_qi_noop_when_sufficient() {
        let wasm = minimal_wasm_qi();
        let mut rt = WasmQiRuntime::new(&wasm).expect("valid wasm");
        // replicas=3, target=3 → should noop (0)
        let result = rt.execute_qi(0, 3, 3).expect("exec");
        assert_eq!(result, 0);
    }

    #[test]
    fn test_reload() {
        let wasm = minimal_wasm_qi();
        let mut rt = WasmQiRuntime::new(&wasm).expect("valid wasm");
        assert!(rt.reload(&wasm).is_ok());
    }
}
