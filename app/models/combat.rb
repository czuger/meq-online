class Combat < ApplicationRecord
  belongs_to :board
  belongs_to :hero, class_name: 'Actor', foreign_key: :actor_id
  belongs_to :mob

  has_many :combat_card_playeds, dependent: :destroy
  has_many :combat_card_played_mobs
  has_many :combat_card_played_heroes

  include GameEngine::CombatMobsPowers

  def reveal_secretly_played_cards
    @game_data_heroes ||= GameData::Heroes.new
    @game_data_mobs_cards ||= GameData::MobsCards.new

    mob.damages_taken_this_turn = 0
    hero.damages_taken_this_turn = 0

    previous_hero_card, previous_mob_card = set_current_and_previous_card
    exhaustion_check

    call_power_params_hero =
        OpenStruct.new( me_previous: previous_hero_card, op_previous: previous_mob_card,
                    op_current: @current_mob_card, me_current: @current_hero_card, me: hero, op: mob, current_combat: self )

    call_power_params_mob =
        OpenStruct.new( me_previous: previous_mob_card, op_previous: previous_hero_card,
                    op_current: @current_hero_card, me_current: @current_mob_card, me: mob, op: hero, current_combat: self )

    call_mob_power( mob, call_power_params_mob )

    previous_hero_card.call_power( :previous, call_power_params_hero ) if previous_hero_card
    previous_mob_card.call_power( :previous, call_power_params_mob ) if previous_mob_card

    @current_hero_card.call_power( :current_cancel, call_power_params_hero )
    @current_mob_card.call_power( :current_cancel, call_power_params_mob )

    @current_hero_card.call_power( :current, call_power_params_hero )
    @current_mob_card.call_power( :current, call_power_params_mob )

    deal_damages

    @current_hero_card.call_power( :after, call_power_params_hero )
    @current_mob_card.call_power( :after, call_power_params_mob )

    hero.save!
    mob.save!

    board.set_hero_activation_state( hero, true ) unless hero_exhausted
    board.set_sauron_activation_state( true ) unless mob_exhausted
  end

  def set_current_and_previous_card
    unless hero_exhausted
      ch_cd = @game_data_heroes.get_card_data( hero.name_code, hero_secret_played_card )
      @current_hero_card = combat_card_played_heroes.create!( card: hero_secret_played_card, pic_path: ch_cd.pic_path, name: ch_cd.name, power: ch_cd.power,
                                                              strength_cost: ch_cd.strength_cost, printed_attack: ch_cd.attack, final_attack: ch_cd.attack,
                                                              printed_defense: ch_cd.defense, final_defense: ch_cd.defense, card_type: ch_cd.type )
    else
      @current_hero_card = create_exhausted_card(combat_card_played_heroes)
    end

    unless mob_exhausted
      cm_cd = @game_data_mobs_cards.get_card_data( mob.attack_deck, mob_secret_played_card )
      @current_mob_card = combat_card_played_mobs.create!( card: mob_secret_played_card, pic_path: cm_cd.pic_path, name: cm_cd.name, power: cm_cd.power,
                                                           strength_cost: cm_cd.strength_cost, printed_attack: cm_cd.attack, final_attack: cm_cd.attack,
                                                           printed_defense: cm_cd.defense, final_defense: cm_cd.defense, card_type: cm_cd.type )
    else
      @current_mob_card = create_exhausted_card(combat_card_played_mobs)
    end

    previous_hero_card = combat_card_played_heroes.where( 'id < ?', @current_hero_card.id ).last
    previous_mob_card = combat_card_played_mobs.where( 'id < ?', @current_mob_card.id ).last

    [previous_hero_card, previous_mob_card]
  end

  def create_exhausted_card( collection )
    collection.create!( card: -1, pic_path: 'Exhausted', name: 'Exhausted', power: 'None',
              strength_cost: 0, printed_attack: 0, final_attack: 0,
              printed_defense: 0, final_defense: 0, card_type: 'none', cancelled: true )
  end

  def deal_damages
    mob_damages = !@current_mob_card.cancelled ? @current_mob_card.final_attack : 0
    hero_damages = !@current_hero_card.cancelled ? @current_hero_card.final_attack : 0

    mob_defense = !@current_mob_card.cancelled ? @current_mob_card.final_defense : 0
    hero_defense = !@current_hero_card.cancelled ? @current_hero_card.final_defense : 0

    mob_damages -= hero_defense
    hero_damages -= mob_defense

    unless @current_mob_card.cancelled
      mob_damages += 1 if mob.code == 'snaga' && @current_mob_card.card_type = 'ranged'
      mob_damages += 1 if mob.code == 'huorn' && @current_mob_card.card_type = 'melee'
    end

    mob_damages = [mob_damages,0].max
    hero_damages = [hero_damages,0].max

    hero.deal_damages( mob_damages ) unless @current_mob_card.cancelled
    mob.deal_damages( hero_damages ) unless @current_hero_card.cancelled
  end
  
  def exhaustion_check
    self.hero_strength_used += @current_hero_card.strength_cost
    if hero_strength_used >= temporary_hero_strength
      @current_hero_card.cancelled = true
      @current_hero_card.save!

      self.hero_exhausted= true
      board.set_hero_activation_state( hero, false )
    end

    mob_strength_cost = @current_mob_card.strength_cost
    mob_strength_cost -= 1 if mob.code == 'southron'
    mob_strength_cost = [ mob_strength_cost, 0 ].max

    self.mob_strength_used += mob_strength_cost
    if mob_strength_used >= mob.strength
      @current_mob_card.cancelled = true
      @current_mob_card.save!

      self.mob_exhausted= true
      board.set_sauron_activation_state( false )
    end

    self.save!
  end

end
