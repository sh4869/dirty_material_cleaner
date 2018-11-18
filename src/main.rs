//! An example of opening an image.
extern crate image;

use std::env;
use std::path::Path;

use image::GenericImageView;

fn is_it_same_x_line(a: (u32, u32), b: (u32, u32)) -> bool {
    return a.0 == b.0 || a.0 == b.0 + 1 || a.0 + 1 == b.0;
}

fn is_it_same_y_line(a: (u32, u32), b: (u32, u32)) -> bool {
    return a.1 == b.1 || a.1 == b.1 + 1 || a.1 + 1 == b.1;
}

fn is_it_same_line(a: (u32, u32), b: (u32, u32)) -> bool {
    return is_it_same_x_line(a, b) || is_it_same_y_line(a, b);
}

// ノイズが交じるのでとりあえず200,200,200以下を白ということに
fn is_white(a: [u8; 4]) -> bool {
    return a[0] > 200 && a[1] > 200 && a[2] > 200;
}

// ノイズが交じるのでとりあえず10,10,10以下を黒ということに
fn is_black(a: [u8; 4]) -> bool {
    return a[0] < 10 && a[1] < 10 && a[2] < 10;
}

// 与えられたpxのposからその4pxの集団の左↑のpxを獲得する
// 何をいっているんだかわからないと思うが画像を拡大すれば言いたいことがわかる
// 他の文字と重なっているときは機能しないが、そもそもピクセルのパターン処理を確認したいだけなので重要じゃない
fn get_left_top_pos_of_4px(target: (u32, u32), im: &image::DynamicImage) -> (u32, u32) {
    let left = target.0 == 0;
    let right = target.0 == im.dimensions().0 - 1;
    let top = target.1 == 0;
    let bottom = target.1 == im.dimensions().1 - 1;
    // 左上が黒のとき
    if !(left || top) && !is_white(im.get_pixel(target.0 - 1, target.1 - 1).data) {
        return (target.0 - 1, target.1 - 1);
    // 右上が黒のとき
    } else if !(right || top) && !is_white(im.get_pixel(target.0 + 1, target.1 - 1).data) {
        return (target.0, target.1 - 1);
    // 左下が黒のとき
    } else if !(left || bottom) && !is_white(im.get_pixel(target.0 - 1, target.1 + 1).data) {
        return (target.0 - 1, target.1);
    }
    return target;
}

fn check_surround_pixel_is_black(target: (u32, u32), im: &image::DynamicImage) -> bool {
    let left = target.0 < 1;
    let right = target.0 + 2 > im.dimensions().0 - 1;
    let top = target.1 < 1;
    let bottom = target.1 + 2 > im.dimensions().1 - 1;
    if left || right || top || bottom {
        return false;
    }
    let left_black = !is_white(im.get_pixel(target.0 - 1, target.1).data) || !is_white(im.get_pixel(target.0 - 1, target.1 + 1).data);
    let right_black = !is_white(im.get_pixel(target.0 + 2, target.1).data) || !is_white(im.get_pixel(target.0 + 2, target.1 + 1).data);
    let top_black = !is_white(im.get_pixel(target.0, target.1 - 1).data) || !is_white(im.get_pixel(target.0 + 1, target.1 - 1).data);
    let bottom_black = !is_white(im.get_pixel(target.0, target.1 + 2).data) || !is_white(im.get_pixel(target.0 + 1, target.1 + 2).data);
    let mut result = vec![left_black, right_black, top_black, bottom_black];
    result.retain(|&v| v);
    return result.iter().count() > 1;
}

fn main() {
    let file = if env::args().count() == 2 {
        env::args().nth(1).unwrap()
    } else {
        panic!("Please enter a file")
    };

    let im = image::open(&Path::new(&file)).unwrap();

    // パターンを記憶する
    let left_pattern = im
        .pixels()
        .filter_map(|p| if is_black(p.2.data) && p.0 == 0 { Some(p) } else { None })
        .map(|p| get_left_top_pos_of_4px((p.0, p.1), &im).1)
        .collect::<Vec<u32>>();

    let top_pattern = im
        .pixels()
        .filter_map(|p| if is_black(p.2.data) && p.1 == 0 { Some(p) } else { None })
        .map(|p| get_left_top_pos_of_4px((p.0, p.1), &im).0)
        .collect::<Vec<u32>>();

    let firstpiexl = im
        .pixels()
        .find_map(|p| if p.2.data != [255, 255, 255, 255] { Some(p) } else { None })
        .unwrap();
    let firstpos = (firstpiexl.0, firstpiexl.1);
    let ypixel = im
        .pixels()
        .find_map(|p| {
            if is_black(p.2.data) && !is_it_same_line(firstpos, (p.0, p.1)) {
                return Some(p);
            } else {
                return None;
            }
        })
        .unwrap();
    let y_next = get_left_top_pos_of_4px((ypixel.0, ypixel.1), &im);

    let second_left_pattern = im
        .pixels()
        .filter_map(|p| {
            if is_black(p.2.data) && is_it_same_x_line((p.0, p.1), y_next) {
                Some(p)
            } else {
                None
            }
        })
        .map(|p| get_left_top_pos_of_4px((p.0, p.1), &im).1)
        .collect::<Vec<u32>>();

    let second_top_pattern = im
        .pixels()
        .filter_map(|p| {
            if is_black(p.2.data) && is_it_same_y_line((p.0, p.1), y_next) {
                Some(p)
            } else {
                None
            }
        })
        .map(|p| get_left_top_pos_of_4px((p.0, p.1), &im).0)
        .collect::<Vec<u32>>();

    // ノイズを消す
    let (width, height) = im.dimensions();
    let mut imgbuf = image::RgbaImage::new(width, height);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let target = im.get_pixel(x, y);
        if is_white(target.data) {
            *pixel = image::Rgba([255, 255, 255, 255]);
        } else {
            *pixel = target;
        }
    }

    for y in 0..height {
        for x in 0..width {
            if (left_pattern.iter().find(|&&l| l == y).is_some() && top_pattern.iter().find(|&&t| t == x).is_some())
                || (second_left_pattern.iter().find(|&&l| l == y).is_some() && second_top_pattern.iter().find(|&&t| t == x).is_some())
            {
                if !check_surround_pixel_is_black((x, y), &im) {
                    imgbuf.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
                    imgbuf.put_pixel(x + 1, y, image::Rgba([255, 255, 255, 255]));
                    imgbuf.put_pixel(x, y + 1, image::Rgba([255, 255, 255, 255]));
                    imgbuf.put_pixel(x + 1, y + 1, image::Rgba([255, 255, 255, 255]));
                }
            }
        }
    }
    // Write the contents of this image to the Writer in PNG format.
    imgbuf.save("test.png").unwrap();
}
