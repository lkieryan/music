import React, { useEffect, useState, useCallback, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { useSearchParams, useNavigate } from 'react-router'
import { useAtom, useAtomValue } from 'jotai'
import { Button } from '~/components/ui/button'

import { Tabs, TabsContent, TabsList, TabsTrigger } from '~/components/ui/tabs'
import {
  searchResultAtom,
  searchLoadingAtom,
  searchErrorAtom,
  lastSearchTermAtom
} from '~/atoms/search'
import { musicSearch } from '~/services/music-api'
import { audioService } from '~/services/audio-service'
import type { Track, Artist, Album, Playlist } from '~/types/sdk-search'
import { sdkTrackToMediaContent } from '~/types/sdk-search'
import { musicIdAtom, musicPlayingAtom } from '~/atoms/player/index'

// Icons
import IconPlay from '~/assets/icons/icon_play.svg?react'
import IconPause from '~/assets/icons/icon_pause.svg?react'
import ReloadIcon from '~/assets/icons/reload-to-stop.svg?react'
import BackIcon from '~/assets/icons/back.svg?react'

// Helper functions
function fixImageUrl(url: string | null): string | null {
  if (!url) return null
  if (url.startsWith('//')) {
    return `https:${url}`
  }
  return url
}

function sanitizeHtml(html: string): string {
  // 简单的HTML清理，只允许基本的格式化标签
  return html
    .replace(/<script[^>]*>.*?<\/script>/gi, '')
    .replace(/<iframe[^>]*>.*?<\/iframe>/gi, '')
    .replace(/javascript:/gi, '')
    .replace(/on\w+="[^"]*"/gi, '')
    .replace(/on\w+='[^']*'/gi, '')
}

function formatDuration(milliseconds?: number | null): string {
  if (!milliseconds) return '--:--'
  const seconds = Math.floor(milliseconds / 1000)
  const mins = Math.floor(seconds / 60)
  const secs = seconds % 60
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
}

export function Component() {
  const { t } = useTranslation(['app', 'common'])
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()

  // Atoms
  const [searchResult, setSearchResult] = useAtom(searchResultAtom)
  const [loadingState, setLoadingState] = useAtom(searchLoadingAtom)
  const [searchError, setSearchError] = useAtom(searchErrorAtom)
  const [lastSearchTerm, setLastSearchTerm] = useAtom(lastSearchTermAtom)


  // Player state
  const currentPlayingId = useAtomValue(musicIdAtom)
  const isGlobalPlaying = useAtomValue(musicPlayingAtom)

  // Local state
  const [activeTab, setActiveTab] = useState<'tracks' | 'artists' | 'albums' | 'playlists'>('tracks')
  const [isLoadingMore, setIsLoadingMore] = useState(false)
  const [currentPage, setCurrentPage] = useState(0)
  const scrollContainerRef = useRef<HTMLDivElement>(null)

  // Get search query from URL params
  const query = searchParams.get('q') || ''

  // Effect to handle search when URL changes but no result in atom
  useEffect(() => {
    const performSearchIfNeeded = async () => {
      // Only search if we don't have results for this query yet
      if (query && query !== lastSearchTerm && loadingState !== 'loading') {
        try {
          setLoadingState('loading')
          setSearchError(null)
          setLastSearchTerm(query)
          setCurrentPage(0) // Reset page counter

          const result = await musicSearch(query, { 
            types: ["All"],
            page: { limit: 50, offset: 0, cursor: null } 
          })
          setSearchResult(result)
          setLoadingState('success')
        } catch (error) {
          console.error('Search failed:', error)
          setSearchError(error instanceof Error ? error.message : t('common:errors.search_failed'))
          setLoadingState('error')
        }
      }
    }

    performSearchIfNeeded()
  }, [query, lastSearchTerm, loadingState, setLoadingState, setSearchError, setSearchResult, setLastSearchTerm, t])

  // Load more results function
  const loadMoreResults = useCallback(async () => {
    if (!query || !searchResult || isLoadingMore || loadingState === 'loading') return

    // Check if current tab has more results
    const currentTabData = searchResult[activeTab]
    if (!currentTabData?.page?.has_more) return

    try {
      setIsLoadingMore(true)
      const nextPage = currentPage + 1
      const nextOffset = nextPage * 50

      const moreResults = await musicSearch(query, {
        types: ["All"],
        page: {
          limit: 50,
          offset: nextOffset,
          cursor: currentTabData.page.next_cursor
        }
      })

      // Merge results with existing ones
      setSearchResult(prev => {
        if (!prev) return moreResults

        return {
          ...prev,
          tracks: {
            items: [...prev.tracks.items, ...moreResults.tracks.items],
            page: moreResults.tracks.page
          },
          artists: {
            items: [...prev.artists.items, ...moreResults.artists.items],
            page: moreResults.artists.page
          },
          albums: {
            items: [...prev.albums.items, ...moreResults.albums.items],
            page: moreResults.albums.page
          },
          playlists: {
            items: [...prev.playlists.items, ...moreResults.playlists.items],
            page: moreResults.playlists.page
          }
        }
      })

      setCurrentPage(nextPage)
    } catch (error) {
      console.error('Load more failed:', error)
    } finally {
      setIsLoadingMore(false)
    }
  }, [query, searchResult, isLoadingMore, loadingState, activeTab, currentPage, setSearchResult])

  // Scroll detection effect
  useEffect(() => {
    const scrollContainer = scrollContainerRef.current
    if (!scrollContainer) return

    const handleScroll = () => {
      const { scrollTop, scrollHeight, clientHeight } = scrollContainer
      const isNearBottom = scrollTop + clientHeight >= scrollHeight - 100 // 100px threshold

      if (isNearBottom && !isLoadingMore) {
        loadMoreResults()
      }
    }

    scrollContainer.addEventListener('scroll', handleScroll)
    return () => scrollContainer.removeEventListener('scroll', handleScroll)
  }, [loadMoreResults, isLoadingMore])

  // Handle back to previous page
  const handleBack = () => {
    navigate(-1)
  }

  // Handle retry search
  const handleRetry = async () => {
    if (!query) return

    try {
      setLoadingState('loading')
      setSearchError(null)
      setLastSearchTerm(query)
      setCurrentPage(0) // Reset page counter

      const result = await musicSearch(query, { 
        types: ["All"],
        page: { limit: 50, offset: 0, cursor: null } 
      })
      setSearchResult(result)
      setLoadingState('success')
    } catch (error) {
      console.error('Retry search failed:', error)
      setSearchError(error instanceof Error ? error.message : t('common:errors.search_failed'))
      setLoadingState('error')
    }
  }

  // Handle play track
  const handlePlayTrack = useCallback(async (track: Track) => {
    try {
      const mediaContent = sdkTrackToMediaContent(track)
      await audioService.playTrack(mediaContent)
    } catch (error) {
      console.error('Failed to play track:', error)
    }
  }, [])

  // Handle play all tracks
  const handlePlayAll = useCallback(async () => {
    try {
      const tracks = searchResult?.tracks?.items || []
      if (tracks.length > 0) {
        const mediaContents = tracks.map(sdkTrackToMediaContent)
        await audioService.playPlaylist(mediaContents, 0)
      }
    } catch (error) {
      console.error('Failed to play all:', error)
    }
  }, [searchResult])

  // Clean up on unmount
  useEffect(() => {
    return () => {
      // Don't clear on unmount - keep data for navigation
    }
  }, [])

  // Loading state
  if (loadingState === 'loading') {
    return (
      <div className="flex flex-col items-center justify-center h-64 space-y-4">
        <ReloadIcon className="w-8 h-8 animate-spin text-blue-500" />
        <p className="text-muted-foreground">{t('common:words.searching')}...</p>
        <p className="text-sm text-muted-foreground">"{query}"</p>
      </div>
    )
  }

  // Error state
  if (loadingState === 'error' || searchError) {
    return (
      <div className="flex flex-col items-center justify-center h-64 space-y-4">
        <div className="text-red-500 text-center">
          <p className="text-lg font-semibold">{t('common:errors.search_failed')}</p>
          <p className="text-sm text-muted-foreground mt-2">{searchError}</p>
        </div>
        <div className="flex gap-3">
          <Button variant="outline" onClick={handleBack}>
            <BackIcon className="w-4 h-4 mr-2" />
            {t('common:actions.back')}
          </Button>
          <Button onClick={handleRetry}>
            <ReloadIcon className="w-4 h-4 mr-2" />
            {t('common:actions.retry')}
          </Button>
        </div>
      </div>
    )
  }

  // No results
  if (!searchResult) {
    return (
      <div className="flex flex-col items-center justify-center h-64 space-y-4">
        <div className="w-12 h-12 bg-muted rounded-full flex items-center justify-center">
          <span className="text-2xl">🎵</span>
        </div>
        <p className="text-muted-foreground">{t('common:words.no_results')}</p>
        <Button variant="outline" onClick={handleBack}>
          <BackIcon className="w-4 h-4 mr-2" />
          {t('common:actions.back')}
        </Button>
      </div>
    )
  }

  const tracks = searchResult?.tracks?.items || []
  const artists = searchResult?.artists?.items || []
  const albums = searchResult?.albums?.items || []
  const playlists = searchResult?.playlists?.items || []

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="border-b border-border px-1 py-4">
        <div className="flex items-center justify-between mb-3">
          <h1 className="text-xl font-bold text-text-primary">
            {t('common:words.search_results')} "{query}"
          </h1>
        </div>

        <div className="flex items-center justify-between">
          <Button
            variant="primary"
            size="sm"
            onClick={handlePlayAll}
            disabled={tracks.length === 0}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
              <path d="M8 5v14l11-7z" fill="currentColor" />
            </svg>
            {t('common:actions.play_all')}
          </Button>

          {/* Tabs moved here */}
          <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as typeof activeTab)}>
            <TabsList>
              <TabsTrigger value="tracks">
                {t('common:words.tracks')} ({tracks.length})
              </TabsTrigger>
              <TabsTrigger value="artists">
                {t('common:words.artists')} ({artists.length})
              </TabsTrigger>
              <TabsTrigger value="albums">
                {t('common:words.albums')} ({albums.length})
              </TabsTrigger>
              <TabsTrigger value="playlists">
                {t('common:words.playlists')} ({playlists.length})
              </TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
      </div>

      {/* Results Content */}
      <div className="flex-1 overflow-hidden">
        <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as typeof activeTab)} className="h-full flex flex-col">

          {/* Tracks Tab */}
          <TabsContent value="tracks" className="flex-1 overflow-hidden">
            {tracks.length > 0 ? (
              <div className="h-full overflow-auto" ref={scrollContainerRef}>
                <table className="table-fixed w-full">
                  <thead>
                    <tr>
                      <th className="w-12 text-left text-sm font-medium text-text-secondary p-2" style={{ width: '48px' }}></th>
                      <th className="text-left text-sm font-medium text-text-secondary p-2">{t('common:words.title')}</th>
                      <th className="text-right text-sm font-medium text-text-secondary p-2" style={{ width: '80px' }}>{t('common:words.duration')}</th>
                      <th className="text-left text-sm font-medium text-text-secondary p-2" style={{ width: '48px' }}></th>
                    </tr>
                  </thead>
                  <tbody>
                    {tracks.map((track, index) => (
                      <tr key={`${track.id}-${index}`} className="group hover:bg-accent/5 border-b border-border/50">
                        <td className="p-2" style={{ width: '48px' }}>
                          <div className="relative w-10 h-10 bg-accent/10 rounded overflow-hidden flex items-center justify-center flex-shrink-0 group">
                            {(() => {
                              const coverUrl = fixImageUrl(track.cover_url)
                              return coverUrl ? (
                                <img
                                  src={coverUrl}
                                  alt=""
                                  className="w-full h-full object-cover rounded"
                                  referrerPolicy="no-referrer"
                                  crossOrigin="anonymous"
                                  onError={(e) => {
                                    e.currentTarget.style.display = 'none'
                                    const parent = e.currentTarget.parentElement
                                    if (parent) {
                                      parent.innerHTML = `
                                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" class="text-accent/60">
                                        <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z" fill="currentColor"/>
                                      </svg>
                                    `
                                    }
                                  }}
                                />
                              ) : (
                                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" className="text-accent/60">
                                  <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z" fill="currentColor" />
                                </svg>
                              )
                            })()}
                            <div className="absolute inset-0 rounded opacity-0 group-hover:opacity-100 transition-opacity duration-200 flex items-center justify-center
                            bg-[rgba(255,255,255,0.04)] dark:bg-[rgba(0,0,0,0.08)] backdrop-blur-[2px] border border-transparent shadow-[inset_0_1px_0_0_rgba(255,255,255,0.06)]">
                              {(() => {
                                const isCurrent = currentPlayingId === track.id
                                const showPause = isCurrent && isGlobalPlaying
                                const onClick = async (e: React.MouseEvent) => {
                                  e.stopPropagation()
                                  try {
                                    if (isCurrent) {
                                      if (isGlobalPlaying) {
                                        await audioService.pause()
                                      } else {
                                        await audioService.play()
                                      }
                                    } else {
                                      await handlePlayTrack(track)
                                    }
                                  } catch (err) {
                                    console.error('change play state failed:', err)
                                  }
                                }
                                return (
                                  <button
                                    onClick={onClick}
                                    className="press-anim-parent text-white hover:text-accent transition-colors duration-200 flex items-center justify-center"
                                  >
                                    {showPause ? (
                                      <IconPause className="w-4 h-4 press-anim" />
                                    ) : (
                                      <IconPlay className="w-4 h-4 press-anim" />
                                    )}
                                  </button>
                                )
                              })()}
                            </div>
                          </div>
                        </td>
                        <td className="p-2">
                          <div className="min-w-0">
                            <div className="flex items-center gap-2">
                              <h3 className="text-sm font-medium text-text-primary truncate">
                                <span dangerouslySetInnerHTML={{
                                  __html: sanitizeHtml(track.title || t('common:words.unknown_title'))
                                }} />
                              </h3>
                            </div>
                            <p className="text-xs text-text-tertiary truncate">
                              {track.artist || t('common:words.unknown_artist')} • {track.album || t('common:words.unknown_album')}
                            </p>
                          </div>
                        </td>
                        <td className="text-right text-text-secondary text-sm p-2" style={{ width: '80px' }}>
                          {formatDuration(track.duration)}
                        </td>
                        <td className="p-2" style={{ width: '48px' }}>
                          <button
                            className="opacity-0 group-hover:opacity-100 p-1 hover:bg-accent/20 rounded transition-all"
                            onClick={(e) => {
                              e.stopPropagation()
                              // TODO: Show context menu
                            }}
                          >
                            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" className="text-text-secondary">
                              <circle cx="12" cy="12" r="1" fill="currentColor" />
                              <circle cx="19" cy="12" r="1" fill="currentColor" />
                              <circle cx="5" cy="12" r="1" fill="currentColor" />
                            </svg>
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>

                {/* Loading more indicator */}
                {isLoadingMore && (
                  <div className="flex items-center justify-center py-4">
                    <ReloadIcon className="w-5 h-5 animate-spin text-blue-500 mr-2" />
                    <span className="text-sm text-muted-foreground">{t('common:words.loading_more')}...</span>
                  </div>
                )}

                {/* End of results indicator */}
                {searchResult?.tracks?.page && !searchResult.tracks.page.has_more && !isLoadingMore && tracks.length > 0 && (
                  <div className="flex items-center justify-center py-4">
                    <span className="text-sm text-muted-foreground">{t('common:words.no_more_results')}</span>
                  </div>
                )}
              </div>
            ) : (
              <div className="flex flex-col items-center justify-center h-64 text-text-secondary">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" className="mb-4 opacity-50">
                  <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z" fill="currentColor" />
                </svg>
                <p className="text-lg font-medium">{t('common:words.no_tracks_found')}</p>
              </div>
            )}
          </TabsContent>

          {/* Artists Tab */}
          <TabsContent value="artists" className="flex-1 overflow-hidden">
            <div className="h-full overflow-auto p-6" ref={activeTab === 'artists' ? scrollContainerRef : undefined}>
              {artists.length > 0 ? (
                <>
                  <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                    {artists.map((artist, index) => (
                      <ArtistCard key={`${artist.id}-${index}`} artist={artist} />
                    ))}
                  </div>

                  {/* Loading more indicator */}
                  {isLoadingMore && activeTab === 'artists' && (
                    <div className="flex items-center justify-center py-4">
                      <ReloadIcon className="w-5 h-5 animate-spin text-blue-500 mr-2" />
                      <span className="text-sm text-muted-foreground">{t('common:words.loading_more')}...</span>
                    </div>
                  )}

                  {/* End of results indicator */}
                  {searchResult?.artists?.page && !searchResult.artists.page.has_more && !isLoadingMore && activeTab === 'artists' && (
                    <div className="flex items-center justify-center py-4">
                      <span className="text-sm text-muted-foreground">{t('common:words.no_more_results')}</span>
                    </div>
                  )}
                </>
              ) : (
                <div className="flex flex-col items-center justify-center h-64 text-text-secondary">
                  <svg width="48" height="48" viewBox="0 0 24 24" fill="none" className="mb-4 opacity-50">
                    <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z" fill="currentColor" />
                  </svg>
                  <p className="text-lg font-medium">{t('common:words.no_artists_found')}</p>
                </div>
              )}
            </div>
          </TabsContent>

          {/* Albums Tab */}
          <TabsContent value="albums" className="flex-1 overflow-hidden">
            <div className="h-full overflow-auto p-6" ref={activeTab === 'albums' ? scrollContainerRef : undefined}>
              {albums.length > 0 ? (
                <>
                  <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                    {albums.map((album, index) => (
                      <AlbumCard key={`${album.id}-${index}`} album={album} />
                    ))}
                  </div>

                  {/* Loading more indicator */}
                  {isLoadingMore && activeTab === 'albums' && (
                    <div className="flex items-center justify-center py-4">
                      <ReloadIcon className="w-5 h-5 animate-spin text-blue-500 mr-2" />
                      <span className="text-sm text-muted-foreground">{t('common:words.loading_more')}...</span>
                    </div>
                  )}

                  {/* End of results indicator */}
                  {searchResult?.albums?.page && !searchResult.albums.page.has_more && !isLoadingMore && activeTab === 'albums' && (
                    <div className="flex items-center justify-center py-4">
                      <span className="text-sm text-muted-foreground">{t('common:words.no_more_results')}</span>
                    </div>
                  )}
                </>
              ) : (
                <div className="flex flex-col items-center justify-center h-64 text-text-secondary">
                  <svg width="48" height="48" viewBox="0 0 24 24" fill="none" className="mb-4 opacity-50">
                    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 14.5c-2.49 0-4.5-2.01-4.5-4.5S9.51 7.5 12 7.5s4.5 2.01 4.5 4.5-2.01 4.5-4.5 4.5zm0-5.5c-.55 0-1 .45-1 1s.45 1 1 1 1-.45 1-1-.45-1-1-1z" fill="currentColor" />
                  </svg>
                  <p className="text-lg font-medium">{t('common:words.no_albums_found')}</p>
                </div>
              )}
            </div>
          </TabsContent>

          {/* Playlists Tab */}
          <TabsContent value="playlists" className="flex-1 overflow-hidden">
            <div className="h-full overflow-auto p-6" ref={activeTab === 'playlists' ? scrollContainerRef : undefined}>
              {playlists.length > 0 ? (
                <>
                  <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                    {playlists.map((playlist, index) => (
                      <PlaylistCard key={`${playlist.id}-${index}`} playlist={playlist} />
                    ))}
                  </div>

                  {/* Loading more indicator */}
                  {isLoadingMore && activeTab === 'playlists' && (
                    <div className="flex items-center justify-center py-4">
                      <ReloadIcon className="w-5 h-5 animate-spin text-blue-500 mr-2" />
                      <span className="text-sm text-muted-foreground">{t('common:words.loading_more')}...</span>
                    </div>
                  )}

                  {/* End of results indicator */}
                  {searchResult?.playlists?.page && !searchResult.playlists.page.has_more && !isLoadingMore && activeTab === 'playlists' && (
                    <div className="flex items-center justify-center py-4">
                      <span className="text-sm text-muted-foreground">{t('common:words.no_more_results')}</span>
                    </div>
                  )}
                </>
              ) : (
                <div className="flex flex-col items-center justify-center h-64 text-text-secondary">
                  <svg width="48" height="48" viewBox="0 0 24 24" fill="none" className="mb-4 opacity-50">
                    <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 2 2h8c1.1 0 2-.9 2-2V8l-6-6zm4 18H6V4h7v5h5v11z" fill="currentColor" />
                  </svg>
                  <p className="text-lg font-medium">{t('common:words.no_playlists_found')}</p>
                </div>
              )}
            </div>
          </TabsContent>
        </Tabs>
      </div>

      {/* Footer */}
      {(tracks.length > 0 || artists.length > 0 || albums.length > 0 || playlists.length > 0) && (
        <div className="border-t border-border px-1 pt-3 text-sm text-text-secondary bg-background-secondary">
          {(() => {
            const currentTotal = tracks.length + artists.length + albums.length + playlists.length
            const totalResults = searchResult?.tracks?.page?.total || 0
            
            if (totalResults > currentTotal) {
              return `${t('common:words.showing')} ${currentTotal} ${t('common:words.of')} ${totalResults} ${t('common:words.results')}`
            } else {
              return `${t('common:words.found')} ${currentTotal} ${t('common:words.results')}`
            }
          })()}
        </div>
      )}
    </div>
  )
}

// Artist card component
function ArtistCard({ artist }: { artist: Artist }) {
  const { t } = useTranslation('common')
  return (
    <div className="group p-4 rounded-lg border border-border hover:bg-accent/5 transition-colors cursor-pointer">
      <div className="flex flex-col items-center text-center">
        <div className="w-16 h-16 bg-accent/10 rounded-full flex items-center justify-center mb-3 overflow-hidden">
          {(() => {
            const avatarUrl = fixImageUrl(artist.avatar_url)
            return avatarUrl ? (
              <img
                src={avatarUrl}
                alt={artist.name || 'Artist'}
                className="w-full h-full object-cover"
                referrerPolicy="no-referrer"
                crossOrigin="anonymous"
                onError={(e) => {
                  e.currentTarget.style.display = 'none'
                  const parent = e.currentTarget.parentElement
                  if (parent) {
                    parent.innerHTML = `
                      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" class="text-accent/60">
                        <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z" fill="currentColor"/>
                      </svg>
                    `
                  }
                }}
              />
            ) : (
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" className="text-accent/60">
                <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z" fill="currentColor" />
              </svg>
            )
          })()}
        </div>
        <h3 className="font-medium text-text-primary truncate w-full">{artist.name || t('common:words.unknown_artist')}</h3>
        {artist.track_count > 0 && (
          <p className="text-sm text-text-secondary">{artist.track_count} {t('common:words.tracks')}</p>
        )}
      </div>
    </div>
  )
}

// Album card component
function AlbumCard({ album }: { album: Album }) {
  const { t } = useTranslation('common')
  return (
    <div className="group p-4 rounded-lg border border-border hover:bg-accent/5 transition-colors cursor-pointer">
      <div className="flex flex-col">
        <div className="aspect-square bg-accent/10 rounded-md flex items-center justify-center mb-3 overflow-hidden">
          {(() => {
            const coverUrl = fixImageUrl(album.cover_url)
            return coverUrl ? (
              <img
                src={coverUrl}
                alt={album.title || 'Album'}
                className="w-full h-full object-cover"
                referrerPolicy="no-referrer"
                crossOrigin="anonymous"
                onError={(e) => {
                  e.currentTarget.style.display = 'none'
                  const parent = e.currentTarget.parentElement
                  if (parent) {
                    parent.innerHTML = `
                      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" class="text-accent/60">
                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 14.5c-2.49 0-4.5-2.01-4.5-4.5S9.51 7.5 12 7.5s4.5 2.01 4.5 4.5-2.01 4.5-4.5 4.5zm0-5.5c-.55 0-1 .45-1 1s.45 1 1 1 1-.45 1-1-.45-1-1-1z" fill="currentColor"/>
                      </svg>
                    `
                  }
                }}
              />
            ) : (
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" className="text-accent/60">
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 14.5c-2.49 0-4.5-2.01-4.5-4.5S9.51 7.5 12 7.5s4.5 2.01 4.5 4.5-2.01 4.5-4.5 4.5zm0-5.5c-.55 0-1 .45-1 1s.45 1 1 1 1-.45 1-1-.45-1-1-1z" fill="currentColor" />
              </svg>
            )
          })()}
        </div>
        <h3 className="font-medium text-text-primary truncate">{album.title || t('common:words.unknown_album')}</h3>
        <p className="text-sm text-text-secondary truncate">{album.artist}</p>
        {album.year && (
          <p className="text-sm text-text-tertiary">{album.year}</p>
        )}
        {album.track_count > 0 && (
          <p className="text-sm text-text-tertiary">{album.track_count} {t('common:words.tracks')}</p>
        )}
      </div>
    </div>
  )
}

// Playlist card component
function PlaylistCard({ playlist }: { playlist: Playlist }) {
  const { t } = useTranslation('common')
  return (
    <div className="group p-4 rounded-lg border border-border hover:bg-accent/5 transition-colors cursor-pointer">
      <div className="flex flex-col">
        <div className="aspect-square bg-accent/10 rounded-md flex items-center justify-center mb-3 overflow-hidden">
          {(() => {
            const coverUrl = fixImageUrl(playlist.cover_url)
            return coverUrl ? (
              <img
                src={coverUrl}
                alt={playlist.title || 'Playlist'}
                className="w-full h-full object-cover"
                referrerPolicy="no-referrer"
                crossOrigin="anonymous"
                onError={(e) => {
                  e.currentTarget.style.display = 'none'
                  const parent = e.currentTarget.parentElement
                  if (parent) {
                    parent.innerHTML = `
                      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" class="text-accent/60">
                        <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 2 2h8c1.1 0 2-.9 2-2V8l-6-6zm4 18H6V4h7v5h5v11z" fill="currentColor"/>
                      </svg>
                    `
                  }
                }}
              />
            ) : (
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" className="text-accent/60">
                <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 2 2h8c1.1 0 2-.9 2-2V8l-6-6zm4 18H6V4h7v5h5v11z" fill="currentColor" />
              </svg>
            )
          })()}
        </div>
        <h3 className="font-medium text-text-primary truncate">{playlist.title || t('common:words.unknown_playlist')}</h3>
        <p className="text-sm text-text-secondary truncate">{playlist.creator}</p>
        {playlist.track_count > 0 && (
          <p className="text-sm text-text-tertiary">{playlist.track_count} {t('common:words.tracks')}</p>
        )}
        {playlist.description && (
          <p className="text-xs text-text-tertiary mt-1 line-clamp-2">{playlist.description}</p>
        )}
      </div>
    </div>
  )
}
