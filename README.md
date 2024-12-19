# **SVD Vector and Linker Script Generator**

This tool scans the current directory for ARM Cortex-M compatible SVD files and automatically generates:

1. **Vector Table File (`vector_<mcu>.txt`)**:
    - Contains the vector table for the specified microcontroller with system exceptions and interrupt handlers.
    - **Format**: A `static VECTOR_TABLE` Rust array with `Option<unsafe fn()>` entries for each vector, including system handlers and IRQs.

2. **Device-Specific Linker Script (`device_<mcu>.x`)**:
    - Defines `PROVIDE` entries for all interrupts as:
      ```text
      PROVIDE(<IRQ_NAME> = default_handler);
      ```
      This facilitates linking during firmware development.

---

## **Usage**

1. **Install the tool using Cargo**:
   ```bash
   cargo install svd-vector-gen


2. **Run the tool**:
    - Ensure that the directory contains valid SVD files.
    - For STM32 microcontrollers, you can obtain SVD files by installing [STM32CubeCLT](https://www.st.com/en/development-tools/stm32cubeclt.html).
    
   ```bash
   svd-vector-gen

## **Example**

For `STM32F303X.svd`:

1. **Generated Files**:
   - `vector_STM32F303X.txt`: Contains the vector table.
   - `device_STM32F303X.x`: Contains the linker script:
     ```text
     PROVIDE(WWDG = default_handler);
     PROVIDE(PVD = default_handler);
     PROVIDE(TAMPER = default_handler);
     ...
     ```


