class Combat < ApplicationRecord
  include AASM

  belongs_to :board
  belongs_to :hero, class_name: 'Actor', foreign_key: :actor_id
  belongs_to :mob

  has_many :combat_card_played_mobs
  has_many :combat_card_played_heroes

  def reveal_secretly_played_cards
    @game_data_heroes ||= GameData::Heroes.new
    @game_data_mobs_cards ||= GameData::MobsCards.new

    ch_cd = @game_data_heroes.get_card_data( hero.name_code, hero_secret_played_card )
    combat_card_played_heroes.create!( card: hero_secret_played_card, pic_path: ch_cd.pic_path, name: ch_cd.name, power: ch_cd.power,
                                       strength_cost: ch_cd.strength_cost, printed_attack: ch_cd.attack, final_attack: ch_cd.attack,
                                       printed_defense: ch_cd.defense, final_defense: ch_cd.defense, card_type: ch_cd.type )

    cm_cd = @game_data_mobs_cards.get_card_data( mob.attack_deck, mob_secret_played_card )
    combat_card_played_mobs.create!( card: mob_secret_played_card, pic_path: cm_cd.pic_path, name: cm_cd.name, power: cm_cd.power,
                                       strength_cost: cm_cd.strength_cost, printed_attack: cm_cd.attack, final_attack: cm_cd.attack,
                                       printed_defense: cm_cd.defense, final_defense: cm_cd.defense, card_type: cm_cd.type )

    #
    # Card cost and eventually exhaustion at this place
    #
    current_hero_card = combat_card_played_heroes.last
    current_mob_card = combat_card_played_mobs.last

    previous_hero_card = combat_card_played_heroes.where( 'id < ?', current_hero_card.id ).last
    previous_mob_card = combat_card_played_mobs.where( 'id < ?', current_mob_card.id ).last

    call_power_params_hero =
        OpenStruct( me_previous: previous_hero_card, op_previous: previous_mob_card,
                    op_current: current_mob_card, me: hero, op: mob, current_combat: self )

    call_power_params_mob =
        OpenStruct( me_previous: previous_mob_card, op_previous: previous_hero_card,
                    op_current: current_hero_card, me: mob, op: hero, current_combat: self )


    previous_hero_card.call_power( :previous, call_power_params_hero ) if previous_hero_card
    previous_mob_card.call_power( :previous, call_power_params_mob ) if previous_mob_card

    current_hero_card.call_power( :before, call_power_params_hero )
    current_mob_card.call_power( :before, call_power_params_mob )

    #
    # Compute combat here
    #

    current_hero_card.call_power( :after, call_power_params_hero )
    current_mob_card.call_power( :after, call_power_params_mob )
  end

end
