use crate::config::pipeline::Area;
use crate::pipeline::Pipeline;
use anyhow::Result;
use ratatui::layout::Layout;
use ratatui::layout::Rect;

impl Pipeline {
    // calculate the layout of the ui widgets based on the terminal size
    pub fn update_ui_rects(&self, terminal_size: Rect) -> Result<()> {
        let entry_area = match self
            .config
            .layout
            .areas
            .iter()
            .find(|a| a.id == self.config.layout.entry)
        {
            Some(area) => area,
            None => {
                panic!("entry area with id {} not found", self.config.layout.entry);
            }
        };
        // recursively iterate through areas in the layout
        let mut area_stack: Vec<Area> = vec![entry_area.clone()];
        let mut rect_stack: Vec<Rect> = vec![terminal_size];

        while !area_stack.is_empty() {
            // handle the area on top of the stack
            let area = match area_stack.pop() {
                Some(area) => area,
                None => continue,
            };
            // get sub areas of this area
            if let Some(constraints) = area.constraints {
                // if constraints is not empty, push sub areas to the stack

                // get the respective rect from the stack
                let rect = match rect_stack.pop() {
                    Some(rect) => rect,
                    None => continue,
                };
                let mut sub_areas: Vec<Area> = vec![];

                // iterate through sub areas id
                for sub_area_id in &constraints {
                    if let Some(sub_area) = self
                        .config
                        .layout
                        .areas
                        .iter()
                        .find(|a| a.id == *sub_area_id)
                    {
                        // push the sub area to the stack
                        area_stack.push(sub_area.clone());
                        // save the sub areas for chunk-splitting later
                        sub_areas.push(sub_area.clone());
                    } else {
                        panic!("sub area with id {} not found", sub_area_id);
                    }
                }
                // get the direction from config
                let direction = match area.direction {
                    Some(dir) => match dir.as_str() {
                        "horizontal" => ratatui::layout::Direction::Horizontal,
                        "vertical" => ratatui::layout::Direction::Vertical,
                        _ => panic!("invalid direction: {}", dir),
                    },
                    None => ratatui::layout::Direction::Vertical,
                };
                // get chunks
                let chunks = Layout::default()
                    .direction(direction)
                    .constraints(
                        sub_areas
                            .iter()
                            .map(|a| ratatui::layout::Constraint::Percentage(a.ratio))
                            .collect::<Vec<_>>(),
                    )
                    .split(rect);

                // push the chunks to the stack
                for chunk in chunks.iter().rev() {
                    rect_stack.push(*chunk);
                }
            } else {
                // this is a leaf area, draw the widget in this area
                let widget_id = match area.widget_id {
                    Some(id) => id,
                    None => {
                        panic!(
                            "area with id {} has no widget_id and isn't a parent area",
                            area.id
                        );
                    }
                };
                if let Some(widget) = self.widgets.get(&widget_id) {
                    // draw the widget in the rect
                    // try to turn widget into Ui traits
                    if let Some(_) = widget.as_ui() {
                        self.ui_rects
                            .lock()
                            .unwrap()
                            .insert(widget_id.clone(), rect_stack.pop().unwrap());
                    } else {
                        panic!("widget with id {} is not a Ui widget", widget_id);
                    }
                } else {
                    panic!("widget with id {} not found", widget_id);
                }
            }
        }
        Ok(())
    }
}
