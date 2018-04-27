require 'rmagick'
require 'fileutils'
card_width=410
card_height=586

name = 'items'
directory = 'items'

FileUtils.mkpath("#{directory}/#{name}")
img = Magick::Image.read("#{directory}/#{name}.jpg")[0]

0.upto(2) do |line|
  0.upto(9) do |col|
    y = line*card_height
    x = col*card_width

    crop = img.crop(x, y, card_width, card_height)
    crop.write("#{directory}/#{name}/#{line}_#{col}.jpg")
  end
end



