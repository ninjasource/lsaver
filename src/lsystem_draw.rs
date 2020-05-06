use super::*;

pub fn draw_lsystem_substring<G>(
    string: &String,
    angle: f64,
    turtle_state: &mut TurtleState,
    params: &Parameters,
    context: Context,
    graphics: &mut G,
) where
    G: Graphics,
{
    use graphics::*;

    let mut current_angle = turtle_state.pos.angle;
    let mut x = turtle_state.pos.x;
    let mut y = turtle_state.pos.y;

    for ch in string.chars() {
        match ch {
            'F' => {
                let mut distance_remaining = params.distance_per_movement;
                let mut next_movement = get_next_pen_movement(
                    x,
                    y,
                    current_angle,
                    distance_remaining,
                    WINDOW_WIDTH,
                    WINDOW_HEIGHT,
                );

                while next_movement.length < distance_remaining {
                    line_from_to(
                        turtle_state.colour,
                        1.0,
                        [x, y],
                        [next_movement.x, next_movement.y],
                        context.transform,
                        graphics,
                    );
                    match next_movement.move_to_x {
                        Some(move_to_x) => x = move_to_x,
                        None => break,
                    };

                    match next_movement.move_to_y {
                        Some(move_to_y) => y = move_to_y,
                        None => break,
                    };

                    distance_remaining -= next_movement.length;
                    next_movement = get_next_pen_movement(
                        x,
                        y,
                        current_angle,
                        distance_remaining,
                        WINDOW_WIDTH,
                        WINDOW_HEIGHT,
                    )
                }

                line_from_to(
                    turtle_state.colour,
                    params.line_width,
                    [x, y],
                    [next_movement.x, next_movement.y],
                    context.transform,
                    graphics,
                );

                x = match next_movement.move_to_x {
                    Some(move_to_x) => move_to_x,
                    None => next_movement.x,
                };

                y = match next_movement.move_to_y {
                    Some(move_to_y) => move_to_y,
                    None => next_movement.y,
                };
            }
            '+' => current_angle += angle,
            '-' => current_angle -= angle,
            '[' => turtle_state.position_stack.push(Position {
                x,
                y,
                angle: current_angle,
            }),
            ']' => {
                // this creates those tree-like patterns
                if let Some(state) = turtle_state.position_stack.pop() {
                    x = state.x;
                    y = state.y;
                    current_angle = state.angle;
                }
            }
            _ => {} // do nothing
        };
    }

    turtle_state.pos = Position {
        x,
        y,
        angle: current_angle,
    };
}

// FIXME: remove duplication in this function
fn get_next_pen_movement(
    x: f64,
    y: f64,
    angle: f64,
    distance: f64,
    max_x: f64,
    max_y: f64,
) -> PossibleMovement {
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();
    let mut new_x = x + cos_angle * distance;
    let mut new_y = y + sin_angle * distance;
    let too_left = new_x < 0.0;
    let too_right = new_x > max_x;
    let too_up = new_y < 0.0;
    let too_down = new_y > max_y;

    let mut dist_until_out_of_bounds;
    let mut possible_movements = vec![PossibleMovement {
        x: new_x,
        y: new_y,
        length: distance,
        move_to_x: None,
        move_to_y: None,
    }];

    if too_right || too_left {
        let move_to_x;
        if too_right {
            dist_until_out_of_bounds = (max_x - x) / cos_angle;
            move_to_x = 0.0;
        } else {
            dist_until_out_of_bounds = -x / cos_angle;
            move_to_x = max_x;
        }

        new_x = x + cos_angle * dist_until_out_of_bounds;
        new_y = y + sin_angle * dist_until_out_of_bounds;

        possible_movements.push(PossibleMovement {
            x: new_x,
            y: new_y,
            length: dist_until_out_of_bounds,
            move_to_x: Some(move_to_x),
            move_to_y: Some(new_y),
        });
    }

    if too_down || too_up {
        let move_to_y;
        if too_down {
            dist_until_out_of_bounds = (max_y - y) / sin_angle;
            move_to_y = 0.0;
        } else {
            dist_until_out_of_bounds = -y / sin_angle;
            move_to_y = max_y;
        }

        new_x = x + cos_angle * dist_until_out_of_bounds;
        new_y = y + sin_angle * dist_until_out_of_bounds;

        possible_movements.push(PossibleMovement {
            x: new_x,
            y: new_y,
            length: dist_until_out_of_bounds,
            move_to_x: Some(new_x),
            move_to_y: Some(move_to_y),
        });
    }

    let mut min_possible_movement = possible_movements.first().unwrap();
    for possible_movement in possible_movements.iter() {
        if possible_movement.length < min_possible_movement.length {
            min_possible_movement = possible_movement
        }
    }

    *min_possible_movement
}

#[derive(Copy, Clone)]
struct PossibleMovement {
    x: f64,
    y: f64,
    length: f64,
    move_to_x: Option<f64>,
    move_to_y: Option<f64>,
}
