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

        // Create the output for the VECTOR_TABLE array
        let mut vector_table_string = "static VECTOR_TABLE: [Option<unsafe fn()>; ".to_string();
        vector_table_string.push_str(&full_vector_table.len().to_string());
        vector_table_string.push_str("] = [\n");

        for entry in full_vector_table {
            vector_table_string.push_str(&format!("    {},\n", entry));
        }

        vector_table_string.push_str("];\n");

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
            .write_all(vector_table_string.as_bytes())
            .expect("Failed to write to vector output file");

        println!("Generated {}", vector_output_path);

        // Create the device-specific linker script with PROVIDE entries
        let mut device_entries = String::new();
        for interrupt in &interrupts {
            device_entries.push_str(&format!("PROVIDE({} = default_handler);\n", interrupt.name));
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
