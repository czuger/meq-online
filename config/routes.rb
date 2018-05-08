Rails.application.routes.draw do

  resources :combats, only: [ :show, :new, :create, :destroy ] do
    get :hero_setup_new
    post :hero_setup_draw_cards
    post :hero_setup_increase_strength

    get :play_card_screen
    post :play_card
  end

  resources :heros, only: [ :index, :show ] do
    post :draw_cards
    get :rest
    get :heal
    patch :take_damages
    post :move
  end

  resources :boards, only:[ :index, :show, :new, :create ] do

    get :join, action: :join_new
    post :join

    resource :logs, only: [:show ]
    resource :sauron, only: [:show ]

    # TODO : extract outside boards, but index
    resources :heros, only: [ :index, :show ] do
      post :draw_cards
      get :rest
      get :heal
      patch :take_damages
      post :move
    end

  end
  # For details on the DSL available within this file, see http://guides.rubyonrails.org/routing.html

  root 'boards#index'

  match 'auth/:provider/callback', to: 'sessions#create', via: [:get, :post]
  get 'auth/failure', to: 'sessions#failure'

  resource :sessions, only: [:new, :destroy]

end
