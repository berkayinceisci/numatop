use crate::app::App;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

pub fn draw(app: &App, frame: &mut Frame) {
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
    let constraints: Vec<Constraint> =
        std::iter::repeat(Constraint::Percentage(100 / num_nodes as u16))
            .take(num_nodes)
            .collect();

    let node_chunks = Layout::horizontal(constraints).split(frame.area());

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
                let cpu_items: Vec<ListItem> = cpus
                    .iter()
                    .map(|cpu| {
                        ListItem::new(format!("Core {}: {:.1}%", cpu.id, /*utilization*/ 50))
                    })
                    .collect();
                let cpu_list = List::new(cpu_items)
                    .block(Block::default().borders(Borders::NONE))
                    .style(Style::default().fg(Color::White));
                frame.render_widget(cpu_list, cpu_list_area);
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
        let memory_section_block = Block::default().title("Memory Utilization");
        let memory_area = inner_chunks[1];
        frame.render_widget(memory_section_block.clone(), memory_area);

        let gauge_area = Layout::default()
            .margin(1)
            .constraints([Constraint::Min(0)])
            .split(memory_area)[0];

        let memory_ratio = if node_data.total_memory_mb > 0 {
            node_data.used_memory_mb as f64 / node_data.total_memory_mb as f64
        } else {
            0.0
        };
        let memory_label = format!(
            "{:.1}/{:.1} GB ({:.0}%)",
            node_data.used_memory_mb as f64 / 1024.0,
            node_data.total_memory_mb as f64 / 1024.0,
            memory_ratio * 100.0
        );

        let memory_gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(
                Style::default()
                    .fg(Color::Green)
                    .bg(Color::Black)
                    .add_modifier(Modifier::ITALIC),
            )
            .percent((memory_ratio * 100.0) as u16)
            .label(memory_label);
        frame.render_widget(memory_gauge, gauge_area);
        //         let memory_area = inner_chunks[1]; // Use the larger part for memory

        //         let memory_ratio = if node_data.total_memory_mb > 0 {
        //             node_data.used_memory_mb as f64 / node_data.total_memory_mb as f64
        //         } else {
        //             0.0
        //         };
        //         let memory_label = format!(
        //             "{:.1}/{:.1} GiB ({:.0}%)", // Displaying as GiB from MB data
        //             node_data.used_memory_mb as f64 / 1024.0,
        //             node_data.total_memory_mb as f64 / 1024.0,
        //             memory_ratio * 100.0
        //         );

        //         let gauge_color = if memory_ratio > 0.85 { Color::Red }
        //                           else if memory_ratio > 0.65 { Color::Yellow }
        //                           else { Color::Green };

        //         let memory_gauge = Gauge::default()
        //             .block(Block::default().title("Memory Usage")) // No borders for gauge itself if inside main block
        //             .gauge_style(
        //                 Style::default()
        //                     .fg(gauge_color)
        //                     .bg(Color::Black) // Background of the unfilled part
        //                     // .add_modifier(Modifier::ITALIC), // Optional
        //             )
        //             .ratio(memory_ratio.min(1.0).max(0.0)) // Clamp ratio between 0 and 1
        //             .label(memory_label);
        //         frame.render_widget(memory_gauge, memory_area);
    }
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
