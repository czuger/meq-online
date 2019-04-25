desc 'Print board graph'
task :print_board_graph => :environment do


  def print_current_state_transitions_and_deep( _object, states, transitions, events )

    states.each do |state|
      _object.aasm_state = state

      _object.aasm.events(:permitted => true).map(&:name).each do |event_name|
        events << event_name
        _object.aasm_state = state

        # transitions << "#{_object.aasm_state} -> #{event_name}"
        # _object.send( event_name )
        # transitions << "#{event_name} -> #{_object.aasm_state}"

        old_state = _object.aasm_state
        _object.send( event_name )
        transitions << "#{old_state} -> #{_object.aasm_state}"

      end
    end
  end

  def create_graphviz_file( states, _object, filename )
    transitions = []
    events = []
    out_file = File.open( "#{filename}.gviz", 'w' )

    out_file.puts 'digraph G {'
    out_file.puts 'concentrate=true'

    print_current_state_transitions_and_deep( _object,  states, transitions, events )

    transitions.uniq!
    events.uniq!

    states.each do |state|
      out_file.puts "#{state} [shape=box];"
      # out_file.puts "#{state} [style=\"rounded,filled\", shape=diamond]"
    end

    out_file.puts transitions.join( "\n" )

    out_file.puts '}'
  end

  states = Board.aasm.states.map(&:name)
  i = Board.new
  create_graphviz_file( states, i, 'board' )

  command = 'dot board.gviz -Tpng -o board.png'
  puts command

  p system( command )
  p system( command )

end