class RefreshChannel < ApplicationCable::Channel
  def subscribed
    stream_from 'refresh_channel'
  end

  def unsubscribed
    # Any cleanup needed when channel is unsubscribed
  end

  def self.refresh
    ActionCable.server.broadcast :refresh_channel, :refresh
  end
end
