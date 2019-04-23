require 'rmagick'
require 'fileutils'

require_relative 'libs/round'

card_width=410
card_height=586

input_path = 'raw_pics/sauron/corruption.jpg'
output_path = '../app/assets/images/corruption/'

`rm #{output_path}/*`

img = Magick::Image.read(input_path)[0]

card_id = 0

0.upto(2) do |line|
  0.upto(9) do |col|
    y = line*card_height
    x = col*card_width

    crop = img.crop(x, y, card_width, card_height)
    crop.rotate!(-90)
    crop = round(crop)
    crop.write("#{output_path}/#{card_id}.png")
    card_id += 1

    break if card_id >= 16
  end
end



