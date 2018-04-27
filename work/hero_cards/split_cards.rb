require 'rmagick'
require 'fileutils'
card_width=410
card_height=586

name = 'argalad'
FileUtils.mkpath(name)
img = Magick::Image.read("#{name}.jpg")[0]

0.upto(2) do |line|
  0.upto(9) do |col|
    y = line*card_height
    x = col*card_width

    crop = img.crop(x, y, card_width, card_height)
    crop.write("#{name}/#{line}_#{col}.jpg")
  end
end



