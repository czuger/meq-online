Rails.application.routes.draw do

  resources :boards do
    resources :heros do
      get :draw_cards
      get :rest
      get :heal
      patch :take_damages
    end
  end
  # For details on the DSL available within this file, see http://guides.rubyonrails.org/routing.html

  root 'boards#index'
end
