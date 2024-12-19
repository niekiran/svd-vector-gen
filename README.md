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

1. **Place your SVD files** in the project directory.
2. **Run the program** using:
   ```bash
   cargo run

3. **The tool generates**:
   - `vector_<mcu>.txt`: Contains the vector table for each SVD file, including system exceptions and interrupt handlers.
   - `device_<mcu>.x`: Linker script with `PROVIDE` entries for IRQs, such as:
     ```text
     PROVIDE(WWDG = default_handler);
     PROVIDE(PVD = default_handler);
     PROVIDE(TAMPER = default_handler);

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


