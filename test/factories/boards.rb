FactoryBot.define do
  factory :board do
    plot_deck { (3..17).to_a }
    shadow_deck { (0..23).to_a }
    event_deck { (0..13).to_a }

    plot_discard {[]}
    shadow_discard {[]}
    event_discard {[]}

    favors {[]}

    corruption_deck {[]}
    corruption_discard {[]}

    monsters_pool_orange {[]}
    monsters_pool_purple {[]}
    monsters_pool_dark_blue {[]}
    monsters_pool_brown {[]}
    monsters_pool_dark_green {[]}

    influence { GameData::Plots.new.get(0).influence.init }
    shadow_pool { GameData::Plots.new.get(0).influence.shadow_pool }

    characters { {} }

    max_heroes_count { 2 }

    sauron_actions { [ 'command_1' ] }
  end
end
