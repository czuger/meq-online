class ExplorationsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor

  # Show the exploration screen
  def show
    # Need to process that.
    @tokens_at_location = @board.get_tokens_at_location(@actor.location)

    corruption_isolated
  end

  # Update the db according to the exploration result
  def update
    notice = nil

    @board.transaction do
      params[:tokens].each do |type, elements|
        case type
          when 'character'
            elements.each do |character|
              @actor.favor += 2
              @board.characters.delete(character)
              @board.log( @actor, 'exploration.encounter_character', { location_name: @actor.current_location_name, character_name: character } )
            end
            notice = 'Character successfully encountered.'
          when 'favor'
             notice = pick_favors(elements)
          when 'plot'
            # For now we assume that there is only one plot at the location. If there is more than one, we get the first
            plot = @board.current_plots.where( affected_location: @actor.location ).first

            if plot.favor_to_discard <= @actor.favor
              @actor.favor -= plot.favor_to_discard
              plot.destroy!
              @board.log( @actor, 'exploration.plot.remove', { location_name: @actor.current_location_name } )
              notice = 'Plot successfully removed.'
            else
              raise "This shouldn't happens"
            end
        end
      end

      @actor.save!
      @board.save!
    end

    redirect_to hero_exploration_url(@actor), notice: notice
  end

  # Next movement step
  def next_movement
    @board.next_to_hero_movement_screen!
    @board.save!

    redirect_to hero_movement_screen_path(@actor)
  end

  # Finish exploration
  def next_step
    if @board.finish_heroes_turn!(@actor) == :hero_draw_cards_screen
      redirect_to hero_draw_cards_screen_path(@actor)
    else
      RefreshChannel.refresh

      redirect_to boards_path
    end
  end

  private

  def corruption_isolated
    if @actor.isolated?
      characters = @tokens_at_location.select{ |e| e.type == :character }
      @tokens_at_location.reject!{ |e| e.type == :character }

      @board.transaction do
        characters.each do |c|
          @board.log( @actor, 'corruption.isolated', character: c.name )
        end
      end
    end
  end

  def pick_favors(elements)
    notice = 'Favor successfully taken.'

    elements.each do |_|
      if @actor.dispairing? && @actor.favor >= 3
        @board.log( @actor, 'corruption.dispairing' )
        notice = I18n.t( 'logs.show.corruption.dispairing' )
        break
      else
        @actor.favor += 1
        # Ensure that only one favor is removed at one time.
        @board.favors.slice!(@board.favors.index(@actor.location))
        @board.log( @actor, 'exploration.get_favor', { location_name: @actor.current_location_name } )
      end
    end

    notice
  end

  def corruption_dispairing
    if @actor.dispairing?
      favors = @tokens_at_location.select{ |e| e.type == :character }
      @tokens_at_location.reject!{ |e| e.type == :character }

      @board.transaction do
        characters.each do |c|
          @board.log( @actor, 'corruption.isolated', character: c.name )
        end
      end
    end
  end

end
