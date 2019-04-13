require 'rmagick'
require 'fileutils'

require_relative 'libs/round.rb'

in_directory = 'monsters/tokens_raw'
cover_directory = 'monsters/tokens_processed/cover'
monster_directory = 'monsters/tokens_processed/monster'

Dir.entries( in_directory ).each do |f|
  next if f == '..' || f == '.'
  p = in_directory + '/' + f
  color = f.split('-').first

  img = Magick::Image.read(p)[0]
  new_img = img.crop(0, 512, 512, 512)
  new_img = round( new_img, 20 )
  new_img.resize_to_fit!( 48, 48 )
  new_img.write("#{cover_directory}/#{color}.jpg")

  monster = f.split('-').last

  new_img = img.crop(0, 0, 512, 512)
  new_img.rotate!(-90)
  new_img = round( new_img, 20 )
  new_img.resize_to_fit!( 48, 48 )
  new_img.write("#{monster_directory}/#{monster}")
end



