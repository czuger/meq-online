module BoardStepsHelper

  def events_list
    @events_list_data ||= @board.aasm.events(:permitted => true).map{ |e| [e.name.to_s.humanize, e.name] }.sort.reverse
    @events_list_data
  end

end
