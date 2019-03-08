Rails.application.routes.draw do

  get 'map_coordinates/edit'
  post 'map_coordinates/update'

  resources :combats, only: [ :show, :new, :create, :destroy ] do
    get :hero_setup_new
    post :hero_setup_draw_cards
    post :hero_setup_increase_strength

    get :play_card_screen
    post :play_card
  end

  resources :heros, only: [ :show ] do
    post :draw_cards
    get :rest
    get :heal
    patch :take_damages
    post :move
    patch :finish_turn

    resources :movement_preparation_steps, except: [ :show ]
  end

  resources :sauron, only: [:show] do
    post :shadow_cards
  end

  resources :story_tracks, only: [:edit, :update]
  resources :sauron_actions, only: [:edit, :update]

  get 'plot_cards/:actor_id/play_screen', to: 'plot_cards#play_screen', as: 'plot_cards_play_screen'
  get 'plot_cards/:actor_id/draw_screen', to: 'plot_cards#draw_screen', as: 'plot_cards_draw_screen'
  post 'plot_cards/:actor_id/draw', to: 'plot_cards#draw', as: 'plot_cards_draw'

  resources :shadow_cards, only:[:edit, :update]

  # Caution, in this case, the id is not a shadow_pool or influence, but an Actor.id
  resources :shadow_pools, only: [ :edit, :update ]
  resources :influences, only: [ :show, :edit, :update ]

  resources :maps, only: [:edit]
  resources :characters, only: [:edit,:update]
  resources :board_steps, only: [:edit,:update]

  resources :boards, only:[ :index, :new, :create ] do
    get :join, action: :join_new
    post :join

    resource :maps, only: [:show]

    resource :logs, only: [:show ]
    resources :heros, only: [ :index ]

  end
  # For details on the DSL available within this file, see http://guides.rubyonrails.org/routing.html

  root 'boards#index'

  match 'auth/:provider/callback', to: 'sessions#create', via: [:get, :post]
  get 'auth/failure', to: 'sessions#failure'

  resource :sessions, only: [:new, :destroy]

end