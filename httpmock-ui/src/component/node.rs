use epaint::color::Color32;
use egui_nodes::{Context, LinkArgs, NodeArgs, NodeConstructor, PinArgs, PinShape};
struct MyApp {
    ctx: Context,
    links: Vec<(usize, usize)>,
}

pub fn example_graph(ctx: &mut Context, links: &mut Vec<(usize, usize)>, ui: &mut egui::Ui) {
    // add nodes with attributes
    let nodes = vec![
        NodeConstructor::new(
            0,
            NodeArgs {
                outline: Some(Color32::RED),
                ..Default::default()
            },
        )
        .with_origin([50.0, 150.0].into())
        .with_title(|ui| ui.label("Example Node A"))
        .with_input_attribute(
            0,
            PinArgs {
                shape: PinShape::Triangle,
                ..Default::default()
            },
            |ui| ui.label("Input"),
        )
        .with_static_attribute(1, |ui| ui.label("Can't Connect to Me"))
        .with_output_attribute(
            2,
            PinArgs {
                shape: PinShape::TriangleFilled,
                ..Default::default()
            },
            |ui| ui.label("Output"),
        ),
        NodeConstructor::new(1, Default::default())
            .with_origin([225.0, 150.0].into())
            .with_title(|ui| ui.label("Example Node B"))
            .with_static_attribute(3, |ui| ui.label("Can't Connect to Me"))
            .with_output_attribute(4, Default::default(), |ui| ui.label("Output"))
            .with_input_attribute(5, Default::default(), |ui| ui.label("Input")),
    ];

    ctx.show(
        nodes,
        links.iter().enumerate().map(|(i, (start, end))| (i, *start, *end, LinkArgs::default())),
        ui,
    );

    // remove destroyed links
    if let Some(idx) = ctx.link_destroyed() {
        links.remove(idx);
    }

    // add created links
    if let Some((start, end, _)) = ctx.link_created() {
        links.push((start, end))
    }
}