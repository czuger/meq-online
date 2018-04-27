require 'rmagick'
require 'fileutils'

directory = 'hero_cards'
name = 'heros'

Dir.entries( directory + '/' + name ).each do |f|
  break if f == '..'
  p = directory + '/' + name + '/' + f
  p p
  img = Magick::Image.read(p)[0]
  img.rotate!(-90)
  img.write(p)
end



