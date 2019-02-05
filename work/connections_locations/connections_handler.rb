require 'pp'
require 'yaml'

# TODO : il faut refaire un check des localisations, elles ne correspondent pas

class ConnectionsHandler

  PATHS = { s: :sun, d: :desolation, f: :forest, m: :mountains, h: :hills, a: :any }.freeze

  def initialize
    @database = File.exists?( 'connections.yaml' ) ? YAML.load_file( 'connections.yaml' ) : {}
    @database.each_value do |v|
      v[:timestamp] ||= DateTime.now
    end
    @locations = File.open( 'locations.txt', 'r').readlines.map(&:chomp).sort
  end

  def run
    while true
      next_unprocessed_destination = get_next_unprocessed_destination

      puts "Enter area connected with #{next_unprocessed_destination}. Press enter to close area."
      puts 'Already connected with ' + list_existing_connections(next_unprocessed_destination)
      area = nil
      until area
        area = find_area
        puts "Area found : #{area}" unless area == :close
      end

      unless area == :close
        puts "Enter path type : #{PATHS.inspect}"
        path = STDIN.getc
        STDIN.getc
        unless PATHS.has_key?(path.to_sym)
          puts "Can't find path of type : '#{path}'"
          next
        end

        puts 'Enter path difficulty'
        diff = STDIN.getc
        STDIN.getc

        create_database_entry next_unprocessed_destination, area, path, diff
      else
        puts "Closing #{next_unprocessed_destination}"
        puts '*' * 20
        @database[next_unprocessed_destination][:closed] = true
      end

      File.open( 'connections.yaml', 'w' ) do |f|
        f.write @database.to_yaml
      end
    end
  end

  private

  def create_database_entry( source, dest, path, diff )
    @database[source] ||= { closed: false, destinations: [], timestamp: DateTime.now }
    @database[dest] ||= { closed: false, destinations: [], timestamp: DateTime.now }

    @database[source][:destinations] << { dest: dest, path_type: PATHS[path.to_sym].to_s, difficulty: diff.to_i }
    @database[dest][:destinations] << { dest: source, path_type: PATHS[path.to_sym].to_s, difficulty: diff.to_i }
  end

  def get_next_unprocessed_destination
    return 'barad_dur' if @database.empty?

    sorted_db_keys = @database.map{ |k, v| [ v[:timestamp].to_time.to_i, k ] }.sort.map{ |e| e.last }
    # p @database.map{ |k, v| [ v[:timestamp].to_time.to_i, k ] }.sort
    # p sorted_db_keys

    # First we look for unclosed starts
    sorted_db_keys.each do |k|
      return k unless @database[k][:closed]
    end

    # Then we look for the next unprocessed destionation
    sorted_db_keys.each do |k|
      @database[k][:destinations].each do |d|
        return d[:dest] unless @database.has_key?( d[:dest] ) && @database[d[:dest]][:closed]
      end
    end
  end

  def list_existing_connections(area)
    @database[area] ? @database[area][:destinations].map{ |e| "'#{e[:dest]}'" }.join( ', ' ) : ''
  end

  def find_area
    area_start = gets.chomp
    return :close if area_start == ''

    found_locations = []
    @locations.each do |l|
      found_locations << l if l.start_with?(area_start)
    end
    if found_locations.count > 1
      puts "Found multiple locations : #{found_locations.inspect}"
      return nil
    end
    found_locations.first
  end

end

ConnectionsHandler.new.run