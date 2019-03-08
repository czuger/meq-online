module PlotCardsHelper

  def sauron_hand_plot_cards_classes
    classes = %w( big-card draggable-card )

    # default selection is single
    if @selection == :multiple
      classes << 'selectable-card-selection-multiple'
    else
      classes << 'selectable-card-selection-unique'
    end

    classes
  end

end
