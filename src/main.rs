use std::fs;
use std::fs::File;
use std::io::Write;
use svd_parser::{
    parse,
    svd::{Cpu, Device},
};

fn main() {
    // Get the current directory
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    println!("Scanning directory: {:?}", current_dir);

    // Collect all SVD files in the directory
    let svd_files: Vec<_> = fs::read_dir(&current_dir)
        .expect("Failed to read directory")
        .filter_map(|entry| {
            let entry = entry.expect("Failed to get directory entry");
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "svd" {
                    return Some(path);
                }
            }
            None
        })
        .collect();

    // If no SVD files are found, throw an error
    if svd_files.is_empty() {
        eprintln!("No SVD files found. Please place SVD files in the project directory.");
        std::process::exit(1);
    }

    // Known Cortex-M exceptions (other than Reset, NMI, HardFault which are defined)
    // This list includes common exceptions across Cortex-M variants.
    // Adjust this list if needed based on the cores you support.
    let known_exceptions = [
        "MemManage_Handler",
        "BusFault_Handler",
        "UsageFault_Handler",
        "SVCall_Handler",
        "DebugMon_Handler",
        "PendSV_Handler",
        "SysTick_Handler",
    ];

    // Handlers we define at the top, including Default_Handler
    let defined_handlers = [
        "Reset_Handler",
        "NMI_Handler",
        "HardFault_Handler",
        "Default_Handler", // Added Default_Handler
    ];

    // Process each SVD file
    for path in svd_files {
        // Read the SVD file
        let svd_content = fs::read_to_string(&path).expect("Failed to read SVD file");

        // Parse the SVD file
        let device: Device = parse(&svd_content).expect("Failed to parse SVD file");

        // Determine the system exception handlers based on the processor architecture
        let system_exceptions: Vec<String> = match device.cpu {
            Some(Cpu { name, .. }) => match name.as_str() {
                "CM0" | "CM0+" => vec![
                    "Some(Reset_Handler)".to_string(),
                    "Some(NMI_Handler)".to_string(),
                    "Some(HardFault_Handler)".to_string(),
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "Some(SVCall_Handler)".to_string(),
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "Some(PendSV_Handler)".to_string(),
                    "Some(SysTick_Handler)".to_string(),
                ],
                "CM3" | "CM4" | "CM7" => vec![
                    "Some(Reset_Handler)".to_string(),
                    "Some(NMI_Handler)".to_string(),
                    "Some(HardFault_Handler)".to_string(),
                    "Some(MemManage_Handler)".to_string(),
                    "Some(BusFault_Handler)".to_string(),
                    "Some(UsageFault_Handler)".to_string(),
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "Some(SVCall_Handler)".to_string(),
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "Some(PendSV_Handler)".to_string(),
                    "Some(SysTick_Handler)".to_string(),
                ],
                _ => vec![
                    "Some(Reset_Handler)".to_string(),
                    "Some(NMI_Handler)".to_string(),
                    "Some(HardFault_Handler)".to_string(),
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "Some(SVCall_Handler)".to_string(),
                    "None".to_string(), // Reserved
                    "None".to_string(), // Reserved
                    "Some(PendSV_Handler)".to_string(),
                    "Some(SysTick_Handler)".to_string(),
                ],
            },
            None => vec![
                "Some(Reset_Handler)".to_string(),
                "Some(NMI_Handler)".to_string(),
                "Some(HardFault_Handler)".to_string(),
                "None".to_string(), // Reserved
                "None".to_string(), // Reserved
                "None".to_string(), // Reserved
                "None".to_string(), // Reserved
                "None".to_string(), // Reserved
                "None".to_string(), // Reserved
                "None".to_string(), // Reserved
                "Some(SVCall_Handler)".to_string(),
                "None".to_string(), // Reserved
                "None".to_string(), // Reserved
                "Some(PendSV_Handler)".to_string(),
                "Some(SysTick_Handler)".to_string(),
            ],
        };

        // Collect all interrupts from the peripherals
        let mut interrupts = Vec::new();
        for peripheral in &device.peripherals {
            for interrupt in &peripheral.interrupt {
                interrupts.push(interrupt.clone());
            }
        }

        // Determine the size of the vector table (maximum interrupt number + 1)
        let max_interrupt_number = interrupts
            .iter()
            .map(|interrupt| interrupt.value as usize)
            .max()
            .unwrap_or(0);

        let mut vector_table: Vec<String> = vec!["None".to_string(); max_interrupt_number + 1];

        for interrupt in &interrupts {
            vector_table[interrupt.value as usize] = format!("Some({}_Handler)", interrupt.name);
        }

        // Combine system exceptions and interrupt handlers into the final vector table
        let mut full_vector_table = system_exceptions;
        full_vector_table.extend(vector_table.into_iter());

        // Extract handler names
        let mut handler_names: Vec<String> = full_vector_table
            .iter()
            .filter_map(|entry| {
                if entry.starts_with("Some(") && entry.ends_with(")") {
                    let inside = &entry[5..entry.len() - 1];
                    if inside.ends_with("_Handler") {
                        Some(inside.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Deduplicate handler names
        let mut unique_handlers = handler_names.clone();
        unique_handlers.sort();
        unique_handlers.dedup();

        // Separate handlers we define from those we declare
        let (mut handlers_to_define, mut handlers_to_declare): (Vec<_>, Vec<_>) =
            unique_handlers
                .into_iter()
                .partition(|h| defined_handlers.contains(&h.as_str()));

        // **Ensure Default_Handler is defined even if not present in vector_table**
        if defined_handlers.contains(&"Default_Handler")
            && !handlers_to_define.contains(&"Default_Handler".to_string())
        {
            handlers_to_define.push("Default_Handler".to_string());
        }

        // Among handlers_to_declare, separate exceptions and interrupts
        // We consider a handler an exception if it's known or appears in the system_exceptions set
        let mut exceptions_to_declare = Vec::new();
        let mut irqs_to_declare = Vec::new();

        for h in handlers_to_declare {
            // If it's a known exception (from known_exceptions) and not defined above, it's an exception
            if known_exceptions.contains(&h.as_str()) {
                exceptions_to_declare.push(h);
            } else {
                // Otherwise it's likely an IRQ
                irqs_to_declare.push(h);
            }
        }

        // Build the top definitions for Reset, NMI, HardFault, and Default_Handler with #[no_mangle]
        let mut top_definitions = String::new();
        for handler in &handlers_to_define {
            top_definitions.push_str(&format!(
                "#[no_mangle]\nextern \"C\" fn {}() {{ loop {{}} }}\n",
                handler
            ));
        }
        top_definitions.push('\n');

        // Build the extern "C" block for the exceptions (except the top 3) first, then interrupts
        let mut extern_block = String::from("extern \"C\" {\n");
        for handler in &exceptions_to_declare {
            extern_block.push_str(&format!("    fn {}();\n", handler));
        }
        for handler in &irqs_to_declare {
            extern_block.push_str(&format!("    fn {}();\n", handler));
        }
        extern_block.push_str("}\n\n");

        // Create the output for the VECTOR_TABLE array with #[used] and #[link_section = ".isr_vector"]
        let mut vector_table_string = String::new();
        vector_table_string.push_str("#[used]\n");
        vector_table_string.push_str("#[link_section = \".isr_vector\"]\n");
        vector_table_string.push_str("static VECTOR_TABLE: [Option<unsafe extern \"C\" fn()>; ");
        vector_table_string.push_str(&full_vector_table.len().to_string());
        vector_table_string.push_str("] = [\n");

        for entry in &full_vector_table {
            vector_table_string.push_str(&format!("    {},\n", entry));
        }

        vector_table_string.push_str("];\n");

        // Combine top_definitions, extern_block, and vector_table_string
        let mut final_vector_file_content = String::new();
        final_vector_file_content.push_str(&top_definitions);
        final_vector_file_content.push_str(&extern_block);
        final_vector_file_content.push_str(&vector_table_string);

        // Generate the output file name based on the SVD file name
        let file_stem = path
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("unknown");
        let vector_output_path = format!("vector_{}.txt", file_stem);

        // Write the vector table to the output file
        let mut vector_file =
            File::create(&vector_output_path).expect("Failed to create vector output file");
        vector_file
            .write_all(final_vector_file_content.as_bytes())
            .expect("Failed to write to vector output file");

        println!("Generated {}", vector_output_path);

        // Create the device-specific linker script with PROVIDE entries

        let mut device_entries = String::new();

        // **First, PROVIDE entries for known exceptions**
        for exception in &known_exceptions {
            device_entries.push_str(&format!(
                "PROVIDE({} = Default_Handler);\n",
                exception
            ));
        }

        // **Then, PROVIDE entries for all other interrupt handlers**
        for interrupt in &irqs_to_declare {
            device_entries.push_str(&format!(
                "PROVIDE({} = Default_Handler);\n",
                interrupt
            ));
        }

        let device_output_path = format!("device_{}.x", file_stem);
        let mut device_file =
            File::create(&device_output_path).expect("Failed to create device output file");
        device_file
            .write_all(device_entries.as_bytes())
            .expect("Failed to write to device output file");

        println!("Generated {}", device_output_path);
    }
}
