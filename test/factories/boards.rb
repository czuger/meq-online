FactoryBot.define do
  factory :board do
    plot_deck { (3..17).to_a }
    shadow_deck { (0..23).to_a }
    event_deck { (0..13).to_a }

    plot_discard {[]}
    shadow_discard {[]}

    influence { GameData::Plots.new.get(0).influence.init }
    shadow_pool { GameData::Plots.new.get(0).influence.shadow_pool }

    current_plots { { 'plot-card-1' => 0 } }

    characters { {} }

    max_heroes_count { 2 }
  end
end
