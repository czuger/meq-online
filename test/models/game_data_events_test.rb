require 'test_helper'
require 'pp'

class GameDataEventsTest < ActiveSupport::TestCase

  test 'check if all event data is correct' do

    characters = GameData::Characters.new
    locations = GameData::Locations.new

    g = GameData::Events.new

    g.data[:I].values.each do |v|

      # p v[:character]

      if v[:character]
        unless characters.exist? v[:character].name
          puts "#{v[:character]} does not exist."
        end

        unless locations.exist? v[:character].location
          puts "#{v[:character]} does not exist."
        end
      end

      v[:favors].each do |favor_location|
        unless locations.exist? favor_location
          puts "#{favor_location} does not exist."
        end
      end
    end
  end
end