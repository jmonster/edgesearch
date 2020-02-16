import {useEffect, useRef, useState} from 'react';
import * as edgesearch from 'edgesearch-client';
import {IFullArticle} from '../FullArticle/FullArticle';
import {IBasicArticle} from '../BasicArticle/BasicArticle';

type EdgesearchResult = string;

const client = new edgesearch.Client<EdgesearchResult>('wiki.wlin.workers.dev');

type WikipediaPageSummary = {
  id: number,
  title: string,
  titleHtml: string,
  url: string,
  image?: string,
  extract: string,
  extractHtml: string,
};

export type FulfilledSearchResults = {
  fullArticles: readonly IFullArticle[],
  basicArticles: readonly IBasicArticle[],
  count: number,
  more: boolean,
};

export const useSearch = () => {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<FulfilledSearchResults | undefined>(undefined);
  const searchDebounce = useRef<number | undefined>();

  useEffect(() => {
    const terms = query.split(/[^a-zA-Z0-9]+/).filter(t => t).map(t => t.toLowerCase());
    setResults(undefined);
    clearTimeout(searchDebounce.current);
    const searchId = searchDebounce.current = window.setTimeout(async () => {
      const titles = await client.search(new edgesearch.Query().add(edgesearch.Mode.REQUIRE, ...terms));
      if (searchId !== searchDebounce.current) {
        return;
      }
      const articles: readonly (WikipediaPageSummary | null)[] = await Promise.all(titles.results.map(async (title) => {
        const res = await fetch(`https://en.wikipedia.org/api/rest_v1/page/summary/${title}`);
        if (res.status < 200 || res.status > 299) {
          return null;
        }
        const metadata = await res.json();
        return {
          id: metadata.pageid,
          title: metadata.title,
          titleHtml: metadata.displaytitle,
          url: metadata.content_urls.desktop.page,
          image: metadata.thumbnail?.source,
          extract: metadata.extract,
          extractHtml: metadata.extract_html,
        };
      }));

      const articleIds = new Set<number>();
      const fullArticles = [];
      const basicArticles = [];
      for (const article of articles) {
        if (!article || articleIds.has(article.id)) {
          continue;
        }
        articleIds.add(article.id);
        if (article.image) {
          fullArticles.push(article as IFullArticle);
        } else {
          basicArticles.push(article as IBasicArticle);
        }
      }
      setResults({
        fullArticles,
        basicArticles,
        count: fullArticles.length + basicArticles.length,
        more: titles.more,
      });
    }, 250);
  }, [query]);

  return {
    query, setQuery, results,
  };
};