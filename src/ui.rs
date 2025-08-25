use crate::app::App;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph},
};

pub fn draw(app: &mut App, frame: &mut Frame) {
    // Clear CPU core areas at the start of each draw
    app.clear_cpu_core_areas();

    let num_nodes = app.numa_nodes.len();
    if num_nodes == 0 {
        frame.render_widget(
            Paragraph::new("No NUMA nodes found or error fetching data.")
                .block(Block::default().title("NUMA Monitor").borders(Borders::ALL)),
            frame.area(),
        );
        return;
    }

    // Create a layout with one column per NUMA node
    // TODO: add layouts.toml file under config/ to allow configuration of runtime layouts
    let constraints: Vec<Constraint> =
        std::iter::repeat(Constraint::Percentage(100 / num_nodes as u16))
            .take(num_nodes)
            .collect();

    let node_chunks = Layout::horizontal(constraints).split(frame.area());

    // Collect all CPU core areas before adding them to app
    let mut all_cpu_core_areas = Vec::new();

    for (i, node_data) in app.numa_nodes.iter().enumerate() {
        let node_chunk = node_chunks[i];
        let node_block = Block::default()
            .title(format!("NUMA Node {}", node_data.id))
            .borders(Borders::ALL);
        frame.render_widget(node_block, node_chunk);

        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .split(node_chunk);

        // --- CPU Utilization Section ---
        let cpu_section_block = Block::default().title("CPU Utilization");
        let cpu_area = inner_chunks[0];
        frame.render_widget(cpu_section_block.clone(), cpu_area);

        let cpu_list_area = Layout::default()
            .margin(1)
            .constraints([Constraint::Min(0)])
            .split(cpu_area)[0];

        if let Some(cpus) = &node_data.cpus {
            if !cpus.is_empty() {
                // Determine number of columns based on CPU count
                let num_cpus = cpus.len();
                let num_columns = if num_cpus <= 24 {
                    1
                } else if num_cpus <= 48 {
                    2
                } else if num_cpus <= 72 {
                    3
                } else {
                    4
                };

                let column_constraints: Vec<Constraint> =
                    vec![Constraint::Percentage(100 / num_columns as u16); num_columns];
                let column_chunks = Layout::horizontal(column_constraints).split(cpu_list_area);
                let items_per_column = (num_cpus as f64 / num_columns as f64).ceil() as usize;

                // Collect CPU core areas first
                let mut node_cpu_core_areas = Vec::new();

                for col in 0..num_columns {
                    let start_idx = col * items_per_column;
                    let end_idx = (start_idx + items_per_column).min(num_cpus);

                    if start_idx >= num_cpus {
                        break;
                    }
                    let column_cpu_items: Vec<ListItem> = cpus[start_idx..end_idx]
                        .iter()
                        .enumerate()
                        .map(|(item_idx, cpu)| {
                            let util_color = if cpu.utilization > 85.0 {
                                Color::Red
                            } else if cpu.utilization > 65.0 {
                                Color::Yellow
                            } else if cpu.utilization > 30.0 {
                                Color::Green
                            } else {
                                Color::Blue
                            };

                            // Calculate the area for this CPU core item
                            let column_area = column_chunks[col];
                            let item_height = 1; // Each ListItem takes 1 row
                            let item_area = Rect {
                                x: column_area.x,
                                y: column_area.y + item_idx as u16,
                                width: column_area.width,
                                height: item_height,
                            };

                            // Store CPU core area for later registration
                            node_cpu_core_areas.push((cpu.id, item_area));

                            let line = Line::from(vec![
                                Span::raw(format!("Core {}: ", cpu.id)),
                                Span::styled(
                                    format!("{:.1}%", cpu.utilization),
                                    Style::default().fg(util_color),
                                ),
                            ]);

                            ListItem::new(line)
                        })
                        .collect();

                    let cpu_list = List::new(column_cpu_items)
                        .block(Block::default().borders(Borders::NONE))
                        .style(Style::default().fg(Color::White));

                    frame.render_widget(cpu_list, column_chunks[col]);
                }

                // Add CPU core areas to the global collection
                all_cpu_core_areas.extend(node_cpu_core_areas);
            } else {
                frame.render_widget(
                    Paragraph::new("No CPUs on this node.")
                        .style(Style::default().fg(Color::Yellow)),
                    cpu_list_area,
                );
            }
        } else {
            frame.render_widget(
                Paragraph::new("CPU-LESS NUMA").style(Style::default().fg(Color::Yellow)),
                cpu_list_area,
            );
        }

        // --- Memory Utilization Section ---
        let memory_area = inner_chunks[1];

        let memory_ratio = if node_data.total_memory_mb > 0 {
            node_data.used_memory_mb as f64 / node_data.total_memory_mb as f64
        } else {
            0.0
        };
        let memory_label = format!(
            "{:.1}/{:.1} GiB ({:.0}%)", // Displaying as GiB from MB data
            node_data.used_memory_mb as f64 / 1024.0,
            node_data.total_memory_mb as f64 / 1024.0,
            memory_ratio * 100.0
        );

        let gauge_color = if memory_ratio > 0.85 {
            Color::Red
        } else if memory_ratio > 0.65 {
            Color::Yellow
        } else {
            Color::Green
        };

        let memory_gauge = Gauge::default()
            .block(Block::default().title("Memory Usage")) // No borders for gauge itself if inside main block
            .gauge_style(
                Style::default().fg(gauge_color).bg(Color::Black), // Background of the unfilled part
                                                                   // .add_modifier(Modifier::ITALIC), // Optional
            )
            .ratio(memory_ratio.min(1.0).max(0.0)) // Clamp ratio between 0 and 1
            .label(memory_label);
        frame.render_widget(memory_gauge, memory_area);
    }

    // Register all CPU core areas after the loop completes
    for (cpu_id, area) in all_cpu_core_areas {
        app.add_cpu_core_area(cpu_id, area);
    }

    // Render popup if it should be shown
    if app.popup_state.show {
        render_process_popup(frame, app);
    }
}

fn render_process_popup(frame: &mut Frame, app: &App) {
    // Create popup area (60% width, 70% height)
    let popup_area = popup_area(frame.area(), 60, 70);

    // Clear the area
    frame.render_widget(Clear, popup_area);

    // Create the popup block
    let popup_block = Block::default()
        .title(format!(
            "Processes on CPU Core {} (Press ESC to close)",
            app.popup_state.cpu_core_id
        ))
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::White));

    frame.render_widget(popup_block, popup_area);

    // Create inner area for the process list
    let inner_area = Layout::default()
        .margin(1)
        .constraints([Constraint::Min(0)])
        .split(popup_area)[0];

    if app.popup_state.processes.is_empty() {
        // Show message when no processes are found
        let no_processes_msg = Paragraph::new("No processes found with affinity to this CPU core")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(no_processes_msg, inner_area);
    } else {
        // Create process list items
        let process_items: Vec<ListItem> = app
            .popup_state
            .processes
            .iter()
            .map(|process| {
                let line = Line::from(vec![
                    Span::raw(format!("PID {}: ", process.pid)),
                    Span::styled(
                        format!("{}", process.name),
                        Style::default().fg(Color::Cyan),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect();

        let process_list = List::new(process_items)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::White));

        frame.render_widget(process_list, inner_area);
    }
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
