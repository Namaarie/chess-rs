use core::panic;

use cozy_chess::{Board, Move, Piece, Rank, Square};
use iced::widget::canvas::{self, Cache, Canvas, Geometry, Image, Event};
use iced::widget::{container, image, row, text};
use iced::{Element, Fill, Point, Rectangle, Renderer, Theme, mouse, Color, Size};

pub fn main() -> iced::Result {
    iced::application("Chess", VisualBoard::update, VisualBoard::view)
        .window_size(Size {
            width: 1280.0,
            height: 720.0,
        })
        .run()
}

fn coord_to_square(x: usize, y: usize) -> Square {
    Square::index(63 - (y * 8 + (7-x)))
}

fn index_to_coord(index: usize) -> (usize, usize) {
    let x = index % 8;
    let y = 7 - (index / 8);
    (x, y)
}

#[derive(Debug, PartialEq)]
enum State {
    Playing,
    Waiting,
    Promoting,
}

struct VisualBoard {
    tile_size: f32,
    dark_color: Color,
    light_color: Color,
    cache: Cache,
    board: Board,
    selected: Option<Square>,
    promotion_square: Option<Square>,
    state: State,
    hovered_tile: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Clicked(Point),
    CursorMoved(Point),
}

impl VisualBoard {
    fn update(&mut self, message: Message) {
        self.cache.clear();

        match message {
            Message::Clicked(point) => {
                match self.state {
                    State::Playing => {
                        if let (Some(selected_square), Some(new_square)) = (self.selected, self.square_from_point(point)) {
                             // check if move would allow promotion
                            if let Some(piece) = self.board.piece_on(selected_square) {
                                if piece == Piece::Pawn{
                                    // piece is pawn so check its valid moves
                                    let mut is_promotion_move = false;
                                    self.board.generate_moves_for(selected_square.bitboard(), |moves| {
                                        for mv in moves {
                                            if mv.to == new_square && mv.promotion.is_some() {
                                                // this is promotion move
                                                is_promotion_move = true;
                                                return true;
                                            }
                                        }
                                        false
                                    });

                                    if is_promotion_move {
                                        self.promotion_square = Some(new_square);
                                        self.state = State::Promoting;
                                        return;
                                    }
                                }
                            }
                           
                            // if Rank::First.bitboard().has(new_square) || Rank::Eighth.bitboard().has(new_square) 
                            // trying to move selected square to new point
                            let _ = self.board.try_play(Move {
                                from: selected_square,
                                to: new_square,
                                promotion: None,
                            });
                        };
                        self.selected = self.square_from_point(point);
                    },
                    State::Waiting => {},
                    State::Promoting => {
                        if let Some((x, y)) = self.hovered_tile {
                            if (2..=5).contains(&x) && y == 4 {
                                // selected this one
                                // determine what piece
                                let piece = match x {
                                    2 => Piece::Rook,
                                    3 => Piece::Knight,
                                    4 => Piece::Bishop,
                                    5 => Piece::Queen,
                                    _ => panic!("???")
                                };

                                let _ = self.board.try_play(Move {
                                    from: self.selected.unwrap(),
                                    to: self.promotion_square.unwrap(),
                                    promotion: Some(piece),
                                });

                                self.state = State::Playing;
                                self.selected = self.square_from_point(point);
                                self.promotion_square = None;
                            }
                        }
                    },
                }
            },
            Message::CursorMoved(point) => {
                let (square_x, square_y) = self.canvas_coord_to_square_coord(point);
                self.hovered_tile = if square_x >= 8.0 || square_x < 0.0 || square_y >= 8.0  || square_y < 0.0 {
                    None
                } else {
                    Some((square_x as usize, square_y as usize))
                }
            },
        }
    }

    fn view(&self) -> Element<Message> {
        container(
            row![
                Canvas::new(self).width(self.tile_size * 8.0).height(self.tile_size * 8.0),
                text(format!(
                    "
                    selected: {:?}
                    status: {:?}
                    to play: {:?}
                    white can castle: {:?}
                    black can castle: {:?}
                    state: {:?}
                    ",
                    self.selected,
                    self.board.status(),
                    self.board.side_to_move(),
                    self.board.castle_rights(cozy_chess::Color::White).long != None && self.board.castle_rights(cozy_chess::Color::White).short != None,
                    self.board.castle_rights(cozy_chess::Color::Black).long != None && self.board.castle_rights(cozy_chess::Color::Black).short != None,
                    self.state,
                )).size(25),
            ].height(Fill)
        ).into()
    }

    fn canvas_coord_to_square_coord(&self, point: Point) -> (f32, f32) {
        let square_x = (point.x) / self.tile_size;
        let square_y = (point.y) / self.tile_size;

        (square_x, square_y)
    }

    fn square_from_point(&mut self, point: Point) -> Option<Square> {
        let (square_x, square_y) = self.canvas_coord_to_square_coord(point);
        if square_x >= 8.0 || square_x < 0.0 || square_y >= 8.0  || square_y < 0.0 {
            None
        } else {
            Some(coord_to_square(square_x as usize, square_y as usize))
        }
    }
}

impl Default for VisualBoard {
    fn default() -> Self {
        VisualBoard {
            cache: Cache::default(),
            tile_size: 64.0,
            light_color: Color::from_rgb8(250, 207, 207),
            dark_color: Color::from_rgb8(154, 122, 161),
            board: Board::default(),
            selected: None,
            promotion_square: None,
            state: State::Playing,
            hovered_tile: None,
        }
    }
}

impl canvas::Program<Message> for VisualBoard {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let cursor_position = cursor.position_in(bounds)?;

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(button)) => match button {
                mouse::Button::Left => Some(canvas::Action::publish(
                    Message::Clicked(cursor_position)
                )),
                _ => None,
            },
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if self.state == State::Promoting {
                    Some(canvas::Action::publish(Message::CursorMoved(position)))
                } else {
                    None
                }
            },
            _ => None,
        }
        .map(canvas::Action::and_capture)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        // println!("drawing");
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            // draw base board
            for y in 0..8 {
                for x in 0..8 {
                    let top_left = Point::new(x as f32 * self.tile_size, y as f32 * self.tile_size);
                    let size = Size::new(self.tile_size, self.tile_size);

                    let color = if (x+y)%2==0 {
                        self.light_color
                    } else {
                        self.dark_color
                    };

                    frame.fill_rectangle(top_left, size, color);
                }
            }

            // draw selection
            let mut indicated_squares = None;

            if let Some(selected_square) = self.selected {
                self.board.generate_moves_for(selected_square.bitboard(), |pm| {
                    indicated_squares = Some(pm);
                    false
                });
            }

            if let Some(pm) = indicated_squares {
                let bitboard = pm.to;

                for square in bitboard.iter() {
                    let (x, y) = index_to_coord(square as usize);

                    let top_left = Point::new(x as f32 * self.tile_size, y as f32 * self.tile_size);
                    let size = Size::new(self.tile_size, self.tile_size);

                    let color = Color::from_rgba(0.0, 0.0, 1.0, 0.5);

                    frame.fill_rectangle(top_left, size, color);
                }
            }

            // draw pieces
            for y in 0..8 {
                for x in 0..8 {
                    let square = coord_to_square(x, y);
                    if let Some(piece) = self.board.piece_on(square) {
                        let mut img_handle: String = "assets/monochrome/".to_owned();
                        if let Some(color) = self.board.color_on(square) {
                            match color {
                                cozy_chess::Color::White => img_handle += "white/",
                                cozy_chess::Color::Black => img_handle += "black/",
                            }
                        }

                        match piece {
                            cozy_chess::Piece::Pawn => img_handle += "pawn.png",
                            cozy_chess::Piece::Knight => img_handle += "knight.png",
                            cozy_chess::Piece::Bishop => img_handle += "bishop.png",
                            cozy_chess::Piece::Rook => img_handle += "rook.png",
                            cozy_chess::Piece::Queen => img_handle += "queen.png",
                            cozy_chess::Piece::King => img_handle += "king.png",
                        }

                        let img = Image::new(img_handle).filter_method(image::FilterMethod::Nearest).snap(true);

                        frame.draw_image(Rectangle{
                            x: x as f32 * self.tile_size,
                            y: y as f32 * self.tile_size,
                            width: 64.0,
                            height: 64.0,
                        }, img);
                    }
                }
            }

            // if in promotion
            if self.state == State::Promoting {
                for y in 0..8 {
                    for x in 0..8 {
                        let top_left = Point::new(x as f32 * self.tile_size, y as f32 * self.tile_size);
                        let size = Size::new(self.tile_size, self.tile_size);

                        let color = Color::from_rgba(0.0, 0.0, 0.0, 0.9);

                        frame.fill_rectangle(top_left, size, color);
                    }
                }

                // hovered tile
                if let Some((x, y)) = self.hovered_tile {
                    if (2..=5).contains(&x) && y == 4 {
                        let top_left = Point::new(x as f32 * self.tile_size, y as f32 * self.tile_size);
                        let size = Size::new(self.tile_size, self.tile_size);
            
                        let color = Color::from_rgba(0.0, 1.0, 0.0, 0.5);
            
                        frame.fill_rectangle(top_left, size, color);
                    }
                }

                // draw the 4 promotion pieces
                let img_handle: String = "assets/color/neutral/".to_owned();

                let img = Image::new(img_handle.clone()+"rook.png").filter_method(image::FilterMethod::Nearest).snap(true);
                let rect = Rectangle{
                    x: 2 as f32 * self.tile_size,
                    y: 4 as f32 * self.tile_size,
                    width: 64.0,
                    height: 64.0,
                };
                frame.draw_image(rect, img);

                let img = Image::new(img_handle.clone()+"knight.png").filter_method(image::FilterMethod::Nearest).snap(true);
                frame.draw_image(Rectangle{
                    x: 3 as f32 * self.tile_size,
                    y: 4 as f32 * self.tile_size,
                    width: 64.0,
                    height: 64.0,
                }, img);
                
                let img = Image::new(img_handle.clone()+"bishop.png").filter_method(image::FilterMethod::Nearest).snap(true);
                frame.draw_image(Rectangle{
                    x: 4 as f32 * self.tile_size,
                    y: 4 as f32 * self.tile_size,
                    width: 64.0,
                    height: 64.0,
                }, img);

                let img = Image::new(img_handle.clone()+"queen.png").filter_method(image::FilterMethod::Nearest).snap(true);
                frame.draw_image(Rectangle{
                    x: 5 as f32 * self.tile_size,
                    y: 4 as f32 * self.tile_size,
                    width: 64.0,
                    height: 64.0,
                }, img);
            }
        });
        vec![geometry]
    }
}
