fn main() {
    let mut b = freertos_cargo_build::Builder::new();

    // Path to FreeRTOS kernel or set ENV "FREERTOS_SRC" instead
    b.freertos("FreeRTOS-Kernel");
    b.freertos_config("src");
    b.freertos_port(String::from("GCC/ARM_CM4F"));
    b.compile().unwrap_or_else(|e| panic!("{}", e.to_string()));
}
