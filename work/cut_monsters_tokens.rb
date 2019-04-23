require 'rmagick'
require 'fileutils'

require_relative 'libs/round.rb'

in_directory = 'raw_pics/monsters/tokens_raw'
cover_directory = '../app/assets/images/monsters/tokens/covers/'
monster_directory = '../app/assets/images/monsters/tokens/sauron_map/'

COLORS_CONVERSION = { 'brown' => 'brown', 'purple' => 'purple', 'blue' => 'dark_blue', 'green' => 'dark_green',
                      'yellow' => 'orange' }

MONSTER_CONVERSION = { 'oliphant.jpg'=> 'oliphaunt.jpg', 'spider.jpg'=> 'giant_spider.jpg', 'troll.jpg'=> 'cave_troll.jpg',
                       'warg.jpg'=> 'warg_rider.jpg', 'wight.jpg'=> 'barrow_wight.jpg'}

Dir.entries( in_directory ).each do |f|
  next if f == '..' || f == '.'
  p = in_directory + '/' + f
  color = f.split('-').first

  img = Magick::Image.read(p)[0]
  new_img = img.crop(0, 512, 512, 512)
  new_img = round( new_img, 20 )
  new_img.resize_to_fit!( 67, 67 )
  # p color
  new_img.write("#{cover_directory}/monsters_pool_#{COLORS_CONVERSION[color]}.jpg")

  monster = f.split('-').last


  new_img = img.crop(0, 0, 512, 512)
  new_img.rotate!(-90)
  new_img = round( new_img, 20 )
  new_img.resize_to_fit!( 67, 67 )

  monster = MONSTER_CONVERSION[monster] || monster
  new_img.write("#{monster_directory}/#{monster}")
end



