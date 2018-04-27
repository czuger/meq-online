Rails.application.routes.draw do

  resources :boards do
    resources :heros do
      get :draw_cards
    end
  end
  # For details on the DSL available within this file, see http://guides.rubyonrails.org/routing.html

  root 'boards#index'
end
