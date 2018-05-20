require 'rmagick'
require 'fileutils'
card_width=410
card_height=586

input_path = 'hero_cards/quests/standard/'
output_path = '../app/assets/images/hero_cards/quests/standard/'

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



